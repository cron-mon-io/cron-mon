// @generated automatically by Diesel CLI.

diesel::table! {
    monitor (monitor_id) {
        monitor_id -> Uuid,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        name -> Varchar,
        expected_duration -> Int4,
        grace_duration -> Int4,
    }
}
