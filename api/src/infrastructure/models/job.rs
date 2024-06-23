use chrono::NaiveDateTime;
use diesel::prelude::*;
use uuid::Uuid;

use crate::domain::models::job::Job;
use crate::infrastructure::db_schema::job;
use crate::infrastructure::models::monitor::MonitorData;

#[derive(Queryable, Identifiable, Selectable, Insertable, Associations, AsChangeset)]
#[diesel(belongs_to(MonitorData, foreign_key = monitor_id))]
#[diesel(table_name = job)]
#[diesel(primary_key(job_id))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct JobData {
    pub job_id: Uuid,
    pub start_time: NaiveDateTime,
    pub max_end_time: NaiveDateTime,
    pub end_time: Option<NaiveDateTime>,
    pub succeeded: Option<bool>,
    pub output: Option<String>,
    pub monitor_id: Uuid,
}

impl Into<Job> for &JobData {
    fn into(self) -> Job {
        Job::new(
            self.job_id,
            self.start_time,
            self.max_end_time,
            self.end_time,
            self.succeeded,
            self.output.clone(),
        )
        // TODO: Handle this in a better way.
        .unwrap()
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use test_utils::{gen_datetime, gen_uuid};

    use super::{Job, JobData};

    #[test]
    fn test_job_data_into_job() {
        let job_data = JobData {
            job_id: gen_uuid("01a92c6c-6803-409d-b675-022fff62575a"),
            start_time: gen_datetime("2024-04-22T22:43:00"),
            max_end_time: gen_datetime("2024-04-22T22:53:00"),
            end_time: Some(gen_datetime("2024-04-22T22:50:00")),
            succeeded: Some(true),
            output: Some(String::from("Job completed successfully")),
            monitor_id: gen_uuid("41ebffb4-a188-48e9-8ec1-61380085cde3"),
        };

        let job: Job = (&job_data).into();

        assert_eq!(job.job_id, gen_uuid("01a92c6c-6803-409d-b675-022fff62575a"));
        assert_eq!(job.start_time, gen_datetime("2024-04-22T22:43:00"));
        assert_eq!(job.max_end_time, gen_datetime("2024-04-22T22:53:00"));
        assert_eq!(job.end_time, Some(gen_datetime("2024-04-22T22:50:00")));
        assert_eq!(job.succeeded, Some(true));
        assert_eq!(job.output, Some(String::from("Job completed successfully")));
    }
}
