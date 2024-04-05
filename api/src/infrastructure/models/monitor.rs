use diesel::prelude::*;
use serde::Serialize;
use uuid::Uuid;

use crate::infrastructure::db_schema::monitor;

#[derive(Serialize, Queryable, Identifiable, Selectable, Insertable, AsChangeset)]
#[diesel(table_name = monitor)]
#[diesel(primary_key(monitor_id))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Monitor {
    pub monitor_id: Uuid,
    pub name: String,
    pub expected_duration: i32,
    pub grace_duration: i32,
}
