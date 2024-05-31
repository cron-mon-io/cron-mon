// @generated automatically by Diesel CLI.

diesel::table! {
    job (job_id) {
        job_id -> Uuid,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        monitor_id -> Uuid,
        start_time -> Timestamp,
        end_time -> Nullable<Timestamp>,
        output -> Nullable<Text>,
        succeeded -> Nullable<Bool>,
        max_end_time -> Timestamp,
    }
}

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

diesel::joinable!(job -> monitor (monitor_id));

diesel::allow_tables_to_appear_in_same_query!(job, monitor);
