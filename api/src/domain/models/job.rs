use chrono::{Duration, NaiveDateTime, Utc};
use serde::{Serialize, Serializer};
use uuid::Uuid;

use crate::errors::Error;

/// The Job struct represents a monitored job, encapsulating the time it started, the time it
/// finished, the resulting status and any output that it produced.
#[derive(Clone, Debug, PartialEq)]
pub struct Job {
    /// The unique identifier for the Job.
    pub job_id: Uuid,
    /// The time that the Job started.
    pub start_time: NaiveDateTime,
    /// The maximum possible end time for the Job before it is considered late.
    pub max_end_time: NaiveDateTime,
    /// The ending state of the job, if it isn't currently in progress.
    pub end_state: Option<EndState>,
    /// Whether or not a late alert has been sent for this Job.
    pub late_alert_sent: bool,
    /// Whether or not an error alert has been sent for this Job.
    pub error_alert_sent: bool,
}

/// The EndState struct represents the state of a Job when it has finished.
#[derive(Clone, Debug, PartialEq)]
pub struct EndState {
    /// The time that the Job finished.
    pub end_time: NaiveDateTime,
    /// Whether or not the Job finished successfully.
    pub succeeded: bool,
    /// Any output from the Job.
    pub output: Option<String>,
}

impl Job {
    /// Start a Job.
    pub fn start(maximum_duration: u64) -> Result<Self, Error> {
        let now = Utc::now().naive_utc();

        // TODO: This doesn't need to retur a result anymore - we've made invalid jobs compile-time
        // errors, yay for Rust!
        Ok(Self {
            job_id: Uuid::new_v4(),
            start_time: now,
            max_end_time: now + Duration::seconds(maximum_duration as i64),
            end_state: None,
            late_alert_sent: false,
            error_alert_sent: false,
        })
    }

    /// Finish the Job. Note that if the Job isn't currently in progress, this will return an
    /// `Error`.
    pub fn finish(&mut self, succeeded: bool, output: Option<String>) -> Result<(), Error> {
        if !self.in_progress() {
            return Err(Error::JobAlreadyFinished(self.job_id));
        }

        self.end_state = Some(EndState {
            end_time: Utc::now().naive_utc(),
            succeeded,
            output,
        });

        Ok(())
    }

    /// Ascertain whether or not the Job is currently in progress.
    pub fn in_progress(&self) -> bool {
        self.end_state.is_none()
    }

    /// Ascertain wher or not the Job is late.
    pub fn late(&self) -> bool {
        let end_time = if let Some(end_state) = &self.end_state {
            end_state.end_time
        } else {
            Utc::now().naive_utc()
        };

        end_time > self.max_end_time
    }

    /// Get the duration of the Job, if it has finished.
    pub fn duration(&self) -> Option<u64> {
        self.end_state.as_ref().map(|end_state| {
            (end_state.end_time - self.start_time)
                .num_seconds()
                .unsigned_abs()
        })
    }
}

impl Serialize for Job {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        #[derive(Serialize)]
        struct SerializedJob {
            // Standard attributes
            job_id: Uuid,
            start_time: NaiveDateTime,
            end_time: Option<NaiveDateTime>,
            succeeded: Option<bool>,
            output: Option<String>,
            // Computed attributes.
            duration: Option<u64>,
            in_progress: bool,
            late: bool,
        }

        let (end_time, succeeded, output) = if let Some(end_state) = &self.end_state {
            (
                Some(end_state.end_time),
                Some(end_state.succeeded),
                end_state.output.clone(),
            )
        } else {
            (None, None, None)
        };
        SerializedJob {
            job_id: self.job_id,
            start_time: self.start_time,
            end_time,
            succeeded,
            output,
            duration: self.duration(),
            in_progress: self.in_progress(),
            late: self.late(),
        }
        .serialize(serializer)
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use chrono::Duration;
    use rstest::*;
    use serde_json::{json, Value};

    use test_utils::{gen_datetime, gen_relative_datetime};

    use super::{EndState, Error, Job, NaiveDateTime, Uuid};

    #[test]
    fn starting_jobs() {
        let job = Job::start(300).expect("Failed to start job");

        assert_eq!(job.max_end_time - job.start_time, Duration::seconds(300));
        assert_eq!(job.end_state, None);
        assert!(!job.late_alert_sent);
        assert!(!job.error_alert_sent);

        // New jobs should always be in progress.
        assert!(job.in_progress());
    }

    #[test]
    fn finishing_jobs() {
        let mut job = Job::start(300).expect("Failed to start job");

        let result1 = job.finish(true, None);
        assert!(result1.is_ok());
        assert!(!job.in_progress());
        assert!(job.end_state.is_some());
        let end_state = job.end_state.as_ref().unwrap();
        let original_end_time = end_state.end_time;
        assert!(end_state.succeeded);
        assert_eq!(end_state.output, None);
        assert!(!job.late_alert_sent);
        assert!(!job.error_alert_sent);

        // Cannot finish a job again once it's been finished.
        let result2 = job.finish(false, Some("It won't wrong".to_owned()));
        assert_eq!(result2.unwrap_err(), Error::JobAlreadyFinished(job.job_id));
        let end_state = job.end_state.unwrap();
        assert_eq!(end_state.end_time, original_end_time);
        assert!(end_state.succeeded);
        assert_eq!(end_state.output, None);
    }

    #[rstest]
    #[case(None, None, None)]
    #[case(Some(gen_datetime("2024-04-20T20:36:00")), Some(true), Some(330))]
    fn getting_job_duration(
        #[case] end_time: Option<NaiveDateTime>,
        #[case] succeeded: Option<bool>,
        #[case] expected_duration: Option<u64>,
    ) {
        let job = Job {
            job_id: Uuid::new_v4(),
            start_time: gen_datetime("2024-04-20T20:30:30"),
            max_end_time: gen_datetime("2024-04-20T20:40:00"),
            end_state: end_time.map(|end_time| EndState {
                end_time,
                succeeded: succeeded.unwrap(),
                output: None,
            }),
            late_alert_sent: false,
            error_alert_sent: false,
        };

        assert_eq!(job.duration(), expected_duration);
    }

    #[rstest]
    // Jobs still in progress - late checks made against current time.
    #[case(gen_relative_datetime(10), (None, None), false)]
    #[case(gen_relative_datetime(-10), (None, None), true)]
    // Finished Jobs - late checks made against end time.
    #[case(
        gen_relative_datetime(0),
        (Some(gen_relative_datetime(-10)), Some(true)),
        false
    )]
    #[case(
        gen_relative_datetime(0),
        (Some(gen_relative_datetime(10)), Some(true)),
        true
    )]
    fn checking_if_job_is_late(
        #[case] max_end_time: NaiveDateTime,
        #[case] result: (Option<NaiveDateTime>, Option<bool>),
        #[case] expected_late: bool,
    ) {
        let job = Job {
            job_id: Uuid::new_v4(),
            start_time: gen_datetime("2024-04-20T20:30:30"),
            max_end_time,
            end_state: result.0.map(|end_time| EndState {
                end_time,
                succeeded: result.1.unwrap(),
                output: None,
            }),
            late_alert_sent: false,
            error_alert_sent: false,
        };

        assert_eq!(job.late(), expected_late);
    }

    #[test]
    fn serialisation() {
        let job = Job {
            job_id: Uuid::from_str("4987dbd2-cbc6-4ea7-b9b4-0af4abb4c0d3").unwrap(),
            start_time: gen_datetime("2024-04-20T20:30:30"),
            max_end_time: gen_datetime("2024-04-20T20:45:30"),
            end_state: Some(EndState {
                end_time: gen_datetime("2024-04-20T20:40:30"),
                succeeded: true,
                output: None,
            }),
            late_alert_sent: false,
            error_alert_sent: false,
        };

        let serialized = json!({"job": job});
        assert_eq!(
            serialized,
            json!(
                {
                    "job": {
                        "job_id": "4987dbd2-cbc6-4ea7-b9b4-0af4abb4c0d3",
                        "start_time": "2024-04-20T20:30:30",
                        "end_time": "2024-04-20T20:40:30",
                        "succeeded": true,
                        "output": Value::Null,
                        "duration": 600,
                        "in_progress": false,
                        "late": false,
                    }
                }
            )
        );
    }
}
