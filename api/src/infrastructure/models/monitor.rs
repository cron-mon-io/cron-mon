use diesel::prelude::*;
use serde::Serialize;
use uuid::Uuid;

use crate::domain::models::monitor::Monitor;
use crate::infrastructure::db_schema::monitor;
use crate::infrastructure::models::job::JobData;

#[derive(Serialize, Queryable, Identifiable, Selectable, Insertable, AsChangeset)]
#[diesel(table_name = monitor)]
#[diesel(primary_key(monitor_id))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct MonitorData {
    pub monitor_id: Uuid,
    pub name: String,
    pub expected_duration: i32,
    pub grace_duration: i32,
}

impl Into<Monitor> for (MonitorData, Vec<JobData>) {
    fn into(self) -> Monitor {
        Monitor {
            monitor_id: self.0.monitor_id,
            name: self.0.name,
            expected_duration: self.0.expected_duration,
            grace_duration: self.0.grace_duration,
            jobs: self.1.into_iter().map(JobData::into).collect(),
        }
    }
}
