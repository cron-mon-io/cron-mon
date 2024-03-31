use serde::Serialize;
use uuid::Uuid;

#[derive(Serialize)]
pub struct Monitor {
    pub monitor_id: Uuid,
    pub name: String,
    pub expected_duration: u32,
    pub grace_duration: u32,
}
