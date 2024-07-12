use chrono::Duration;
use serde::Serialize;
use uuid::Uuid;

use crate::domain::models::job::Job;
use crate::errors::AppError;

/// The `Monitor` struct represents a Monitor for cron jobs and the like, and is ultimately the core
/// part of the Cron Mon domain.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Monitor {
    /// The unique identifier for the Monitor.
    pub monitor_id: Uuid,
    /// The Monitor's name (typically the command or filename that the cronjob will invoke).
    pub name: String,
    /// The expected duration of the monitored cronjob, in seconds.
    pub expected_duration: i32,
    /// The amount of time, in seconds, to allow the monitored cronjob to overrun by before
    /// considering them late.
    pub grace_duration: i32,
    /// The history of jobs that have been monitored.
    pub jobs: Vec<Job>,
}

impl Monitor {
    /// Instatiate a new Monitor.
    pub fn new(name: String, expected_duration: i32, grace_duration: i32) -> Self {
        Self {
            monitor_id: Uuid::new_v4(),
            name,
            expected_duration,
            grace_duration,
            jobs: vec![],
        }
    }

    /// Modify the Monitor's details.
    pub fn edit_details(&mut self, name: String, expected_duration: i32, grace_duration: i32) {
        self.name = name;
        self.expected_duration = expected_duration;
        self.grace_duration = grace_duration;
    }

    /// Retrieve the jobs currently in progress.
    pub fn jobs_in_progress(&self) -> Vec<&Job> {
        self.jobs.iter().filter(|job| job.in_progress()).collect()
    }

    /// Retrieve late jobs.
    ///
    /// Jobs are considered late once they have been running for more than
    /// `expected_duration + grace_duration`. Note that late Jobs can still finish, either
    /// successfully or in error.
    pub fn late_jobs(&self) -> Vec<&Job> {
        self.jobs.iter().filter(|job| job.late()).collect()
    }

    /// Retrieve the most recently finished job.
    pub fn last_finished_job(&self) -> Option<&Job> {
        self.jobs.iter().find(|&job| job.succeeded.is_some())
    }

    /// Retrieve the most recently started job.
    pub fn last_started_job(&self) -> Option<&Job> {
        // Jobs will be naturally ordered by start_time, so we can just take a reference to the
        // first job - if we have any.
        if !self.jobs.is_empty() {
            Some(&self.jobs[0])
        } else {
            None
        }
    }

    /// Start a new job
    pub fn start_job(&mut self) -> Result<Job, AppError> {
        // We give the job the _current_ maximum duration here so that if the monitor is modified,
        // any previous and in progress jobs are not affected.
        let new_job = Job::start(self.maximum_duration().num_seconds() as u64)?;
        self.jobs.push(new_job.clone());
        Ok(new_job)
    }

    /// Finish a job. Note that this will return an `AppError` is a Job with the given `job_id`
    /// cannot be found in the Monitor, or if the Job isn't currently in progress.
    pub fn finish_job(
        &mut self,
        job_id: Uuid,
        succeeded: bool,
        output: Option<String>,
    ) -> Result<&Job, AppError> {
        let monitor_id = self.monitor_id;
        let job = self.get_job(job_id);
        match job {
            Some(j) => {
                j.finish(succeeded, output)?;
                Ok(j)
            }
            None => Err(AppError::JobNotFound(monitor_id, job_id)),
        }
    }

    /// Retrieve a Job from the Monitor by its Job ID.
    pub fn get_job(&mut self, job_id: Uuid) -> Option<&mut Job> {
        self.jobs.iter_mut().find(|job| job.job_id == job_id)
    }

    fn maximum_duration(&self) -> chrono::TimeDelta {
        Duration::seconds((self.expected_duration + self.grace_duration) as i64)
    }
}

#[cfg(test)]
mod tests {

    use chrono::NaiveDateTime;
    use rstest::rstest;

    use test_utils::{gen_relative_datetime, gen_uuid};

    use super::{AppError, Job, Monitor, Uuid};

    #[test]
    fn creating_new_monitors() {
        let mon = Monitor::new("new-monitor".to_owned(), 3600, 600);

        assert_eq!(mon.name, "new-monitor".to_owned());
        assert_eq!(mon.expected_duration, 3600);
        assert_eq!(mon.grace_duration, 600);
        assert!(mon.jobs_in_progress().is_empty());
        assert!(mon.jobs.is_empty());
    }

    #[rstest]
    #[case(
        vec![
            (
                gen_uuid("79192674-0e87-4f79-b988-0efd5ae76420"), gen_relative_datetime(5)
            ),
            (
                gen_uuid("15904641-2d0e-4d27-8fd0-b130f0ab5aa9"), gen_relative_datetime(5)
            )
        ],
        vec![]
    )]
    #[case(
        vec![
            (
                gen_uuid("79192674-0e87-4f79-b988-0efd5ae76420"), gen_relative_datetime(0)
            ),
            (
                gen_uuid("15904641-2d0e-4d27-8fd0-b130f0ab5aa9"), gen_relative_datetime(5)
            )
        ],
        vec![gen_uuid("79192674-0e87-4f79-b988-0efd5ae76420")]
    )]
    #[case(
        vec![
            (
                gen_uuid("79192674-0e87-4f79-b988-0efd5ae76420"), gen_relative_datetime(0)
            ),
            (
                gen_uuid("15904641-2d0e-4d27-8fd0-b130f0ab5aa9"), gen_relative_datetime(0)
            )
        ],
        vec![
            gen_uuid("79192674-0e87-4f79-b988-0efd5ae76420"),
            gen_uuid("15904641-2d0e-4d27-8fd0-b130f0ab5aa9")
        ]
    )]
    fn checking_for_late_jobs(
        #[case] input: Vec<(Uuid, NaiveDateTime)>,
        #[case] expected_ids: Vec<Uuid>,
    ) {
        let mut mon = Monitor::new("new-monitor".to_owned(), 200, 100);
        mon.jobs = input
            .iter()
            .map(|i| Job {
                job_id: i.0,
                start_time: gen_relative_datetime(-200),
                max_end_time: i.1,
                end_time: None,
                succeeded: None,
                output: None,
            })
            .collect();

        let late_jobs_ids: Vec<Uuid> = mon.late_jobs().iter().map(|job| job.job_id).collect();
        assert_eq!(late_jobs_ids, expected_ids);
    }

    #[test]
    fn getting_the_last_finished_job() {
        let mut mon = Monitor::new("new-monitor".to_owned(), 200, 100);
        mon.jobs = vec![
            Job {
                job_id: gen_uuid("70e7f11b-7ae3-4e69-adb0-52fdbf775ee1"),
                start_time: gen_relative_datetime(0),
                max_end_time: gen_relative_datetime(300),
                end_time: None,
                succeeded: None,
                output: None,
            },
            Job {
                job_id: gen_uuid("139fbf11-eff1-44cf-9f58-b5febb4729d6"),
                start_time: gen_relative_datetime(-200),
                max_end_time: gen_relative_datetime(100),
                end_time: Some(gen_relative_datetime(0)),
                succeeded: Some(true),
                output: None,
            },
            Job {
                job_id: gen_uuid("a4a8d5ac-86c1-448d-aa82-3388d59ac43e"),
                start_time: gen_relative_datetime(-300),
                max_end_time: gen_relative_datetime(0),
                end_time: Some(gen_relative_datetime(-50)),
                succeeded: Some(false),
                output: None,
            },
        ];

        let last_finished_job = mon.last_finished_job().unwrap();
        assert_eq!(
            last_finished_job.job_id,
            gen_uuid("139fbf11-eff1-44cf-9f58-b5febb4729d6")
        );
    }

    #[test]
    fn getting_the_last_finished_job_when_no_jobs_have_finished() {
        let mut mon = Monitor::new("new-monitor".to_owned(), 200, 100);
        mon.jobs = vec![
            Job {
                job_id: gen_uuid("70e7f11b-7ae3-4e69-adb0-52fdbf775ee1"),
                start_time: gen_relative_datetime(0),
                max_end_time: gen_relative_datetime(300),
                end_time: None,
                succeeded: None,
                output: None,
            },
            Job {
                job_id: gen_uuid("139fbf11-eff1-44cf-9f58-b5febb4729d6"),
                start_time: gen_relative_datetime(-200),
                max_end_time: gen_relative_datetime(100),
                end_time: None,
                succeeded: None,
                output: None,
            },
            Job {
                job_id: gen_uuid("a4a8d5ac-86c1-448d-aa82-3388d59ac43e"),
                start_time: gen_relative_datetime(-300),
                max_end_time: gen_relative_datetime(0),
                end_time: None,
                succeeded: None,
                output: None,
            },
        ];

        let last_finished_job = mon.last_finished_job();
        assert!(last_finished_job.is_none());
    }

    #[test]
    fn getting_the_last_started_job() {
        let mut mon = Monitor::new("new-monitor".to_owned(), 200, 100);
        mon.jobs = vec![
            Job {
                job_id: gen_uuid("70e7f11b-7ae3-4e69-adb0-52fdbf775ee1"),
                start_time: gen_relative_datetime(0),
                max_end_time: gen_relative_datetime(300),
                end_time: None,
                succeeded: None,
                output: None,
            },
            Job {
                job_id: gen_uuid("139fbf11-eff1-44cf-9f58-b5febb4729d6"),
                start_time: gen_relative_datetime(-200),
                max_end_time: gen_relative_datetime(100),
                end_time: None,
                succeeded: None,
                output: None,
            },
            Job {
                job_id: gen_uuid("a4a8d5ac-86c1-448d-aa82-3388d59ac43e"),
                start_time: gen_relative_datetime(-300),
                max_end_time: gen_relative_datetime(0),
                end_time: None,
                succeeded: None,
                output: None,
            },
        ];

        let last_started_job = mon.last_started_job().unwrap();
        assert_eq!(
            last_started_job.job_id,
            gen_uuid("70e7f11b-7ae3-4e69-adb0-52fdbf775ee1")
        );
    }

    #[test]
    fn getting_the_last_started_job_when_no_jobs_have_started() {
        let mon = Monitor::new("new-monitor".to_owned(), 200, 100);

        let last_started_job = mon.last_started_job();
        assert!(last_started_job.is_none());
    }

    #[test]
    fn editing_monitors() {
        let mut mon = Monitor::new("new-monitor".to_owned(), 3600, 600);

        mon.edit_details("new-name".to_owned(), 360, 60);

        assert_eq!(mon.name, "new-name".to_owned());
        assert_eq!(mon.expected_duration, 360);
        assert_eq!(mon.grace_duration, 60);
    }

    #[test]
    fn starting_jobs() {
        let mut mon = Monitor::new("new-monitor".to_owned(), 3600, 600);

        assert!(mon.jobs_in_progress().is_empty());

        let job1 = mon.start_job().expect("Failed to start job");
        let job2 = mon.start_job().expect("Failed to start job");
        let job3 = mon.start_job().expect("Failed to start job");

        assert_eq!(mon.jobs_in_progress().len(), 3);

        // Ensure all jobs are genuinely different.
        assert_ne!(job1.job_id, job2.job_id);
        assert_ne!(job1.job_id, job3.job_id);
        assert_ne!(job2.job_id, job3.job_id);
    }

    #[test]
    fn finishing_jobs() {
        let mut mon = Monitor::new("new-monitor".to_owned(), 3600, 600);

        let job1 = mon.start_job().expect("Failed to start job");

        assert_eq!(mon.jobs_in_progress().len(), 1);

        let result1 = mon.finish_job(job1.job_id, true, None);

        assert!(result1.is_ok());
        assert_eq!(mon.jobs_in_progress().len(), 0);

        let result2 = mon.finish_job(
            gen_uuid("4631aa50-7780-455a-ab9a-78292f931832"),
            false,
            None,
        );
        assert_eq!(
            result2.unwrap_err(),
            AppError::JobNotFound(
                mon.monitor_id,
                gen_uuid("4631aa50-7780-455a-ab9a-78292f931832")
            )
        );
    }
}
