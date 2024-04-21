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
    }
}
