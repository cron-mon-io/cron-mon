use diesel::prelude::*;
use uuid::Uuid;

use crate::domain::models::{job::Job, monitor::Monitor};
use crate::errors::AppError;
use crate::infrastructure::db_schema::monitor;
use crate::infrastructure::models::job::JobData;

#[derive(Queryable, Identifiable, Selectable, Insertable, AsChangeset)]
#[diesel(table_name = monitor)]
#[diesel(primary_key(monitor_id))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct MonitorData {
    pub monitor_id: Uuid,
    pub name: String,
    pub expected_duration: i32,
    pub grace_duration: i32,
}

impl MonitorData {
    pub fn to_model(&self, job_datas: &[JobData]) -> Result<Monitor, AppError> {
        Ok(Monitor {
            monitor_id: self.monitor_id,
            name: self.name.clone(),
            expected_duration: self.expected_duration,
            grace_duration: self.grace_duration,
            jobs: job_datas
                .iter()
                .map(|jd| jd.into())
                .collect::<Result<Vec<Job>, AppError>>()?,
        })
    }
}

impl From<&Monitor> for (MonitorData, Vec<JobData>) {
    fn from(value: &Monitor) -> Self {
        (
            MonitorData {
                monitor_id: value.monitor_id,
                name: value.name.clone(),
                expected_duration: value.expected_duration,
                grace_duration: value.grace_duration,
            },
            value
                .jobs
                .iter()
                .map(|job| JobData {
                    job_id: job.job_id,
                    monitor_id: value.monitor_id,
                    start_time: job.start_time,
                    max_end_time: job.max_end_time,
                    end_time: job.end_time,
                    succeeded: job.succeeded,
                    output: job.output.clone(),
                })
                .collect(),
        )
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use test_utils::{gen_datetime, gen_uuid};

    use crate::domain::models::job::Job;

    use super::{JobData, Monitor, MonitorData};

    #[test]
    fn test_monitor_to_db_data() {
        let monitor = Monitor {
            monitor_id: gen_uuid("41ebffb4-a188-48e9-8ec1-61380085cde3"),
            name: "foo".to_owned(),
            expected_duration: 300,
            grace_duration: 100,
            jobs: vec![Job::new(
                gen_uuid("01a92c6c-6803-409d-b675-022fff62575a"),
                gen_datetime("2024-04-22T22:43:00"),
                gen_datetime("2024-04-22T22:53:00"),
                None,
                None,
                None,
            )
            .unwrap()],
        };

        let (monitor_data, job_data) = <(MonitorData, Vec<JobData>)>::from(&monitor);

        assert_eq!(monitor_data.monitor_id, monitor.monitor_id);
        assert_eq!(monitor_data.name, monitor.name);
        assert_eq!(monitor_data.expected_duration, monitor.expected_duration);
        assert_eq!(monitor_data.grace_duration, monitor.grace_duration);

        assert_eq!(job_data.len(), 1);
        let job_data = &job_data[0];
        assert_eq!(
            job_data.job_id,
            gen_uuid("01a92c6c-6803-409d-b675-022fff62575a")
        );
        assert_eq!(
            job_data.monitor_id,
            gen_uuid("41ebffb4-a188-48e9-8ec1-61380085cde3")
        );
        assert_eq!(job_data.start_time, gen_datetime("2024-04-22T22:43:00"));
        assert_eq!(job_data.max_end_time, gen_datetime("2024-04-22T22:53:00"));
        assert_eq!(job_data.end_time, None);
        assert_eq!(job_data.succeeded, None);
        assert_eq!(job_data.output, None);
    }

    #[test]
    fn test_converting_db_to_monitor() {
        let monitor_data = MonitorData {
            monitor_id: gen_uuid("41ebffb4-a188-48e9-8ec1-61380085cde3"),
            name: "foo".to_owned(),
            expected_duration: 300,
            grace_duration: 100,
        };

        let job_data = vec![JobData {
            job_id: gen_uuid("01a92c6c-6803-409d-b675-022fff62575a"),
            monitor_id: gen_uuid("41ebffb4-a188-48e9-8ec1-61380085cde3"),
            start_time: gen_datetime("2024-04-22T22:43:00"),
            max_end_time: gen_datetime("2024-04-22T22:53:00"),
            end_time: None,
            succeeded: None,
            output: None,
        }];

        let monitor = monitor_data.to_model(&job_data).unwrap();

        assert_eq!(
            monitor.monitor_id,
            gen_uuid("41ebffb4-a188-48e9-8ec1-61380085cde3")
        );
        assert_eq!(monitor.name, "foo".to_owned());
        assert_eq!(monitor.expected_duration, 300);
        assert_eq!(monitor.grace_duration, 100);

        assert_eq!(monitor.jobs.len(), 1);
        let job = &monitor.jobs[0];
        assert_eq!(job.job_id, gen_uuid("01a92c6c-6803-409d-b675-022fff62575a"));
        assert_eq!(job.start_time, gen_datetime("2024-04-22T22:43:00"));
        assert_eq!(job.max_end_time, gen_datetime("2024-04-22T22:53:00"));
        assert_eq!(job.end_time, None);
        assert_eq!(job.succeeded, None);
        assert_eq!(job.output, None);
    }
}
