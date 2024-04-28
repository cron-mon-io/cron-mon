pub mod common;

use rocket::http::{ContentType, Status};
use serde_json::{json, Value};

use common::get_test_client;

#[test]
fn test_get_monitors() {
    let client = get_test_client();

    let response = client.get("/api/v1/monitors").dispatch();

    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.content_type(), Some(ContentType::JSON));

    assert_eq!(
        response.into_json::<Value>().unwrap(),
        json!({
          "data": [
            {
              "expected_duration": 1800,
              "grace_duration": 600,
              "monitor_id": "c1bf0515-df39-448b-aa95-686360a33b36",
              "name": "db-backup.py"
            },
            {
              "expected_duration": 5400,
              "grace_duration": 720,
              "monitor_id": "f0b291fe-bd41-4787-bc2d-1329903f7a6a",
              "name": "generate-orders.sh"
            },
            {
              "expected_duration": 900,
              "grace_duration": 300,
              "monitor_id": "a04376e2-0fb5-4949-9744-7c5d0a50b411",
              "name": "init-philanges"
            },
            {
              "expected_duration": 300,
              "grace_duration": 120,
              "monitor_id": "309a68f1-d6a2-4312-8012-49c1b9b9af25",
              "name": "gen-manifests | send-manifest"
            },
            {
              "expected_duration": 10800,
              "grace_duration": 1800,
              "monitor_id": "0798c530-34a4-4452-b2dc-f8140fd498d5",
              "name": "bill-and-invoice"
            }
          ],
          "paging": {
            "total": 5
          }
        })
    );
}
