use diesel::prelude::*;
use uuid::Uuid;

use crate::domain::models::monitor::Monitor;
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

impl Into<Monitor> for (&MonitorData, &Vec<JobData>) {
    fn into(self) -> Monitor {
        // TODO: Test me
        Monitor {
            monitor_id: self.0.monitor_id,
            name: self.0.name.clone(),
            expected_duration: self.0.expected_duration,
            grace_duration: self.0.grace_duration,
            jobs: self.1.iter().map(|jd: &JobData| jd.into()).collect(),
        }
    }
}

impl From<&Monitor> for (MonitorData, Vec<JobData>) {
    fn from(value: &Monitor) -> Self {
        // TODO: Test me
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
