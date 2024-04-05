use chrono::offset::Utc;
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Deserialize, Serialize)]
pub struct Job {
    pub job_id: Uuid,
    pub start_time: NaiveDateTime,
    pub end_time: Option<NaiveDateTime>,
    pub status: Option<String>,
    pub output: Option<String>,
}

impl Job {
    pub fn finish(&mut self, status: String, output: Option<String>) {
        self.status = Some(status);
        self.output = output;
        self.end_time = Some(Utc::now().naive_utc());
    }

    pub fn in_progress(&self) -> bool {
        self.end_time.is_none()
    }
}
