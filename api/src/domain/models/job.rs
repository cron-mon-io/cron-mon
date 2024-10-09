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
    /// The time that the job finished, if it isn't currently in progress.
    pub end_time: Option<NaiveDateTime>,
    /// Whether or not the Job finished successfully, if it isn't currently in progress.
    pub succeeded: Option<bool>,
    /// Any output from the Job, if it isn't currently in progress.
    pub output: Option<String>,
}

impl Job {
    /// Construct a Job instance.
    pub fn new(
        job_id: Uuid,
        start_time: NaiveDateTime,
        max_end_time: NaiveDateTime,
        end_time: Option<NaiveDateTime>,
        succeeded: Option<bool>,
        output: Option<String>,
    ) -> Result<Self, Error> {
        // Job's must either have no end_time or succeeded, or both.
        if (end_time.is_some() || succeeded.is_some())
            && (end_time.is_none() || succeeded.is_none())
        {
            return Err(Error::InvalidJob("Job is in an invalid state".to_owned()));
        }
        Ok(Job {
            job_id,
            start_time,
            max_end_time,
            end_time,
            succeeded,
            output,
        })
    }

    /// Start a Job.
    pub fn start(maximum_duration: u64) -> Result<Self, Error> {
        let now = Utc::now().naive_utc();

        Job::new(
            Uuid::new_v4(),
            now,
            now + Duration::seconds(maximum_duration as i64),
            None,
            None,
            None,
        )
    }

    /// Finish the Job. Note that if the Job isn't currently in progress, this will return an
    /// `Error`.
    pub fn finish(&mut self, succeeded: bool, output: Option<String>) -> Result<(), Error> {
        if !self.in_progress() {
            return Err(Error::JobAlreadyFinished(self.job_id));
        }

        self.succeeded = Some(succeeded);
        self.output = output;
        self.end_time = Some(Utc::now().naive_utc());

        Ok(())
    }

    /// Ascertain whether or not the Job is currently in progress.
    pub fn in_progress(&self) -> bool {
        self.end_time.is_none()
    }

    /// Ascertain wher or not the Job is late.
    pub fn late(&self) -> bool {
        let end_time = if let Some(end_time) = self.end_time {
            end_time
        } else {
            Utc::now().naive_utc()
        };

        end_time > self.max_end_time
    }

    /// Get the duration of the Job, if it has finished.
    pub fn duration(&self) -> Option<u64> {
        if !self.in_progress() {
            Some(
                (self.end_time.unwrap() - self.start_time)
                    .num_seconds()
                    .unsigned_abs(),
            )
        } else {
            None
        }
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

        SerializedJob {
            job_id: self.job_id,
            start_time: self.start_time,
            end_time: self.end_time,
            succeeded: self.succeeded,
            output: self.output.clone(),
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

    use super::{Error, Job, NaiveDateTime, Uuid};

    #[test]
    fn starting_jobs() {
        let job = Job::start(300).expect("Failed to start job");

        assert_eq!(job.max_end_time - job.start_time, Duration::seconds(300));
        assert_eq!(job.end_time, None);
        assert_eq!(job.succeeded, None);
        assert_eq!(job.output, None);

        // New jobs should always be in progress.
        assert!(job.in_progress());
    }

    #[test]
    fn finishing_jobs() {
        let mut job = Job::start(300).expect("Failed to start job");

        let result1 = job.finish(true, None);
        assert!(result1.is_ok());
        assert!(!job.in_progress());
        assert!(job.end_time.is_some());
        assert_eq!(job.succeeded, Some(true));
        assert_eq!(job.output, None);

        // Cannot finish a job again once it's been finished.
        let result2 = job.finish(false, Some("It won't wrong".to_owned()));
        assert_eq!(result2.unwrap_err(), Error::JobAlreadyFinished(job.job_id));
        assert_eq!(job.succeeded, Some(true));
        assert_eq!(job.output, None);
    }

    #[rstest]
    #[case(None, None, None)]
    #[case(Some(gen_datetime("2024-04-20T20:36:00")), Some(true), Some(330))]
    fn getting_job_duration(
        #[case] end_time: Option<NaiveDateTime>,
        #[case] succeeded: Option<bool>,
        #[case] expected_duration: Option<u64>,
    ) {
        let job = Job::new(
            Uuid::new_v4(),
            gen_datetime("2024-04-20T20:30:30"),
            gen_datetime("2024-04-20T20:40:00"),
            end_time,
            succeeded,
            None,
        )
        .expect("Failed to create job");

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
        let job = Job::new(
            Uuid::new_v4(),
            gen_datetime("2024-04-20T20:30:30"),
            max_end_time,
            result.0,
            result.1,
            None,
        )
        .expect("Failed to create job");

        assert_eq!(job.late(), expected_late);
    }

    #[test]
    fn validation() {
        let job = Job::new(
            Uuid::new_v4(),
            gen_datetime("2024-04-20T20:30:30"),
            gen_datetime("2024-04-20T20:40:30"),
            Some(gen_datetime("2024-04-20T20:35:30")),
            None,
            None,
        );

        assert!(job.is_err());
        assert_eq!(
            job.unwrap_err(),
            Error::InvalidJob("Job is in an invalid state".to_owned())
        );
    }

    #[test]
    fn serialisation() {
        let job = Job::new(
            Uuid::from_str("4987dbd2-cbc6-4ea7-b9b4-0af4abb4c0d3").unwrap(),
            gen_datetime("2024-04-20T20:30:30"),
            gen_datetime("2024-04-20T20:45:30"),
            Some(gen_datetime("2024-04-20T20:40:30")),
            Some(true),
            None,
        )
        .unwrap();

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
