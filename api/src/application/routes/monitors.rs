use rocket::serde::json::{json, Json, Value};
use serde::Deserialize;
use uuid::{uuid, Uuid};

use crate::domain::models::monitor::Monitor;
use crate::infrastructure::paging::Paging;

#[derive(Deserialize)]
pub struct NewMonitor {
    name: String,
    expected_duration: u32,
    grace_duration: u32,
}

#[get("/monitors")]
pub fn list_monitors() -> Value {
    json![{
        "data": vec![
            Monitor {
                monitor_id: uuid!["1ae45ad1-6972-4ea7-b4c4-93be7893ae3e"],
                name: "foo".to_owned(),
                expected_duration: 1234,
                grace_duration: 30,
            },
            Monitor {
                monitor_id: uuid!["be1d0d3f-ff8c-4294-9b42-521108deeae6"],
                name: "bar".to_owned(),
                expected_duration: 5678,
                grace_duration: 60,
            },
        ],
        "paging": Paging { total: 2 },
    }]
}

#[post("/monitors", data = "<monitor>")]
pub fn create_monitor(monitor: Json<NewMonitor>) -> Value {
    json![{
        "data": Monitor {
            monitor_id: Uuid::new_v4(),
            name: monitor.name.clone(),
            expected_duration: monitor.expected_duration,
            grace_duration: monitor.grace_duration
        }
    }]
}

#[get("/monitors/<monitor_id>")]
pub fn get_monitor(monitor_id: Uuid) -> Value {
    json![{
        "data": Monitor {
            monitor_id,
            name: "foo".to_owned(),
            expected_duration: 1234,
            grace_duration: 30,
        }
    }]
}
