use diesel::prelude::*;
use serde::Serialize;
use uuid::Uuid;

#[derive(Serialize, Queryable, Selectable)]
#[diesel(table_name = crate::infrastructure::db_schema::monitor)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Monitor {
    pub monitor_id: Uuid,
    pub name: String,
    pub expected_duration: i32,
    pub grace_duration: i32,
}
