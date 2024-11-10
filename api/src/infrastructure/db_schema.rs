// @generated automatically by Diesel CLI.

diesel::table! {
    api_key (api_key_id) {
        api_key_id -> Uuid,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        key -> Varchar,
        tenant -> Varchar,
        last_used -> Nullable<Timestamp>,
        last_used_monitor_id -> Nullable<Uuid>,
        last_used_monitor_name -> Nullable<Varchar>,
        name -> Varchar,
        masked -> Varchar,
    }
}

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
        tenant -> Varchar,
    }
}

diesel::joinable!(job -> monitor (monitor_id));

diesel::allow_tables_to_appear_in_same_query!(api_key, job, monitor,);
