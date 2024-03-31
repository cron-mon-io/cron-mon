use rocket::serde::json::Json;
use serde::Serialize;
use uuid::uuid;

use crate::domain::models::monitor::Monitor;
use crate::infrastructure::paging::Paging;

#[derive(Serialize)]
pub struct MonitorList {
    data: Vec<Monitor>,
    paging: Paging,
}

#[get("/monitors")]
pub fn list_monitors() -> Json<MonitorList> {
    Json(MonitorList {
        data: vec![
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
        paging: Paging { total: 2 },
    })
}
