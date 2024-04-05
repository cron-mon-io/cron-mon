use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::Serialize;
use uuid::Uuid;

use crate::infrastructure::db_schema::job;
use crate::infrastructure::models::monitor::Monitor;

#[derive(Serialize, Queryable, Identifiable, Selectable, Associations, AsChangeset)]
#[diesel(belongs_to(Monitor))]
#[diesel(table_name = job)]
#[diesel(primary_key(job_id))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Job {
    pub job_id: Uuid,
    pub start_time: NaiveDateTime,
    pub end_time: Option<NaiveDateTime>,
    pub status: Option<String>,
    pub output: Option<String>,
    pub monitor_id: Uuid,
}
