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
    pub end_time: Option<NaiveDateTime>,
    pub status: Option<String>,
    pub output: Option<String>,
    pub monitor_id: Uuid,
}

impl Into<Job> for JobData {
    fn into(self) -> Job {
        Job {
            job_id: self.job_id,
            start_time: self.start_time,
            end_time: self.end_time,
            status: self.status,
            output: self.output,
        }
    }
}
