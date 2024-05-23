use diesel_async::AsyncPgConnection;
use diesel_async::RunQueryDsl;
use rocket::local::blocking::Client;
use tokio;

use test_utils::{gen_datetime, gen_uuid};

use cron_mon_api::infrastructure::database::establish_connection;
use cron_mon_api::infrastructure::db_schema::job;
use cron_mon_api::infrastructure::db_schema::monitor;
use cron_mon_api::infrastructure::models::{job::JobData, monitor::MonitorData};
use cron_mon_api::rocket;

pub async fn setup_db() -> AsyncPgConnection {
    // TODO: Find a nice way of putting these seeds in their own file, having them here is a bit
    // messy to say the least.
    let monitor_seeds: Vec<MonitorData> = vec![
        MonitorData {
            monitor_id: gen_uuid("a04376e2-0fb5-4949-9744-7c5d0a50b411"),
            name: "init-philanges".to_string(),
            expected_duration: 900,
            grace_duration: 300,
        },
        MonitorData {
            monitor_id: gen_uuid("c1bf0515-df39-448b-aa95-686360a33b36"),
            name: "db-backup.py".to_string(),
            expected_duration: 1800,
            grace_duration: 600,
        },
        MonitorData {
            monitor_id: gen_uuid("f0b291fe-bd41-4787-bc2d-1329903f7a6a"),
            name: "generate-orders.sh".to_string(),
            expected_duration: 5400,
            grace_duration: 720,
        },
    ];
    let job_seeds: Vec<JobData> = vec![
        JobData {
            job_id: gen_uuid("8106bab7-d643-4ede-bd92-60c79f787344"),
            monitor_id: gen_uuid("c1bf0515-df39-448b-aa95-686360a33b36"),
            start_time: gen_datetime("2024-05-01T00:10:00.000"),
            max_end_time: gen_datetime("2024-05-01T00:50:00.000"),
            end_time: Some(gen_datetime("2024-05-01T00:49:00.000")),
            succeeded: Some(true),
            output: Some("Database successfully backed up".to_string()),
        },
        JobData {
            job_id: gen_uuid("c1893113-66d7-4707-9a51-c8be46287b2c"),
            monitor_id: gen_uuid("c1bf0515-df39-448b-aa95-686360a33b36"),
            start_time: gen_datetime("2024-05-01T00:00:00.000"),
            max_end_time: gen_datetime("2024-05-01T00:40:00.000"),
            end_time: Some(gen_datetime("2024-05-01T00:39:00.000")),
            succeeded: Some(false),
            output: Some("Could not connect to database".to_string()),
        },
        JobData {
            job_id: gen_uuid("9d4e2d69-af63-4c1e-8639-60cb2683aee5"),
            monitor_id: gen_uuid("c1bf0515-df39-448b-aa95-686360a33b36"),
            start_time: gen_datetime("2024-05-01T00:20:00.000"),
            max_end_time: gen_datetime("2024-05-01T01:00:00.000"),
            end_time: None,
            succeeded: None,
            output: None,
        },
        JobData {
            job_id: gen_uuid("2a09c819-ed8c-4e3a-b085-889f3f475c02"),
            monitor_id: gen_uuid("f0b291fe-bd41-4787-bc2d-1329903f7a6a"),
            start_time: gen_datetime("2024-05-01T00:00:00.000"),
            max_end_time: gen_datetime("2024-05-01T00:42:00.000"),
            end_time: None,
            succeeded: None,
            output: None,
        },
        JobData {
            job_id: gen_uuid("db610603-5094-49a4-8838-204103cd5b78"),
            monitor_id: gen_uuid("f0b291fe-bd41-4787-bc2d-1329903f7a6a"),
            start_time: gen_datetime("2024-05-01T00:00:00.000"),
            max_end_time: gen_datetime("2024-05-01T00:42:00.000"),
            end_time: None,
            succeeded: None,
            output: None,
        },
    ];

    let mut conn = establish_connection().await;

    diesel::delete(monitor::table)
        .execute(&mut conn)
        .await
        .expect("Failed to existing data");

    diesel::insert_into(monitor::table)
        .values(&monitor_seeds)
        .execute(&mut conn)
        .await
        .expect("Failed to seed monitors");

    diesel::insert_into(job::table)
        .values(&job_seeds)
        .execute(&mut conn)
        .await
        .expect("Failed to seed jobs");

    conn
}

pub fn get_test_client(seed_db: bool) -> Client {
    if seed_db {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(async {
                setup_db().await;
                ()
            })
    }
    Client::tracked(rocket()).expect("Invalid rocket instance")
}
