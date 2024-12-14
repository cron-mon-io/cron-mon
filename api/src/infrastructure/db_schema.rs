// @generated automatically by Diesel CLI.

diesel::table! {
    alert_config (alert_config_id) {
        alert_config_id -> Uuid,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        name -> Varchar,
        tenant -> Varchar,
        #[sql_name = "type"]
        type_ -> Varchar,
        active -> Bool,
        on_late -> Bool,
        on_error -> Bool,
    }
}

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
        late_alert_sent -> Bool,
        error_alert_sent -> Bool,
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

diesel::table! {
    monitor_alert_config (alert_config_id, monitor_id) {
        alert_config_id -> Uuid,
        monitor_id -> Uuid,
        monitor_name -> Varchar,
    }
}

diesel::table! {
    slack_alert_config (alert_config_id) {
        alert_config_id -> Uuid,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        slack_channel -> Varchar,
        slack_bot_oauth_token -> Varchar,
    }
}

diesel::joinable!(job -> monitor (monitor_id));
diesel::joinable!(monitor_alert_config -> alert_config (alert_config_id));
diesel::joinable!(monitor_alert_config -> monitor (monitor_id));
diesel::joinable!(slack_alert_config -> alert_config (alert_config_id));

diesel::allow_tables_to_appear_in_same_query!(
    alert_config,
    api_key,
    job,
    monitor,
    monitor_alert_config,
    slack_alert_config,
);
