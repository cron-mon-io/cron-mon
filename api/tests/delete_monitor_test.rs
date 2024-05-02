pub mod common;

use std::format;

use rocket::http::Status;
use rstest::*;

use common::{get_num_monitors, get_test_client};

#[rstest]
#[case("c1bf0515-df39-448b-aa95-686360a33b36", Status::Ok, -1)]
#[case("cc6cf74e-b25d-4c8c-94a6-914e3f139c14", Status::NotFound, 0)]
#[test]
fn test_delete_monitor_deletes(
    #[case] monitor_id: &str,
    #[case] status: Status,
    #[case] adjustment: i64,
) {
    let client = get_test_client(true);

    // Get starting number of monitors.
    let num_monitors = get_num_monitors(&client);

    let response = client
        .delete(format!("/api/v1/monitors/{}", monitor_id))
        .dispatch();

    assert_eq!(response.status(), status);

    // Ensure we definitely have - or haven't - deleted a monitor.
    assert_eq!(get_num_monitors(&client), num_monitors + adjustment);
}
