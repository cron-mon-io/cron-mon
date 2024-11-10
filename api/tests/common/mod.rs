use std::time::{SystemTime, UNIX_EPOCH};

use diesel_async::RunQueryDsl;
use rocket::http::Header;
use rocket::local::asynchronous::Client;
use serde_json::json;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use test_utils::{encode_jwt, gen_datetime, gen_uuid, RSA_EXPONENT, RSA_MODULUS};

use cron_mon_api::infrastructure::auth::Jwt;
use cron_mon_api::infrastructure::database::{create_connection_pool, DbPool};
use cron_mon_api::infrastructure::db_schema::{api_key, job, monitor};
use cron_mon_api::infrastructure::models::{
    api_key::ApiKeyData, job::JobData, monitor::MonitorData,
};
use cron_mon_api::rocket;

pub async fn setup_db_pool() -> DbPool {
    let (monitor_seeds, job_seeds, api_key_seeds) = seed_data();
    seed_db(&monitor_seeds, &job_seeds, &api_key_seeds).await
}

pub async fn get_test_client(kid: &str, seed_db: bool) -> (MockServer, Client) {
    let mock_server = setup_mock_jwks_server(kid).await;
    if seed_db {
        setup_db_pool().await;
    }
    let client = Client::tracked(rocket())
        .await
        .expect("Invalid rocket instance");

    (mock_server, client)
}

pub fn seed_data() -> (Vec<MonitorData>, Vec<JobData>, Vec<ApiKeyData>) {
    (
        vec![
            MonitorData {
                monitor_id: gen_uuid("a04376e2-0fb5-4949-9744-7c5d0a50b411"),
                tenant: "foo".to_owned(),
                name: "init-philanges".to_string(),
                expected_duration: 900,
                grace_duration: 300,
            },
            MonitorData {
                monitor_id: gen_uuid("c1bf0515-df39-448b-aa95-686360a33b36"),
                tenant: "foo".to_owned(),
                name: "db-backup.py".to_string(),
                expected_duration: 1800,
                grace_duration: 600,
            },
            MonitorData {
                monitor_id: gen_uuid("f0b291fe-bd41-4787-bc2d-1329903f7a6a"),
                tenant: "foo".to_owned(),
                name: "generate-orders.sh".to_string(),
                expected_duration: 5400,
                grace_duration: 720,
            },
            MonitorData {
                monitor_id: gen_uuid("cc6cf74e-b25d-4c8c-94a6-914e3f139c14"),
                tenant: "bar".to_owned(),
                name: "data-snapshot.py".to_string(),
                expected_duration: 3600,
                grace_duration: 1200,
            },
        ],
        vec![
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
        ],
        vec![
            ApiKeyData {
                api_key_id: gen_uuid("bfab6d41-8b00-49ef-86df-f562b701ee4f"),
                created_at: gen_datetime("2024-05-01T00:00:00.000"),
                tenant: "foo".to_owned(),
                name: "Test foo key".to_string(),
                key: "104e4587f5340bd9264ea0fee2075627c74420bd5c48aa9e8a463f03a2675020".to_string(),
                masked: "foo-k************-key".to_string(),
                last_used: Some(gen_datetime("2024-11-01T00:00:00.000")),
                last_used_monitor_id: Some(gen_uuid("c1bf0515-df39-448b-aa95-686360a33b36")),
                last_used_monitor_name: Some("db-backup.py".to_string()),
            },
            ApiKeyData {
                api_key_id: gen_uuid("029d7c3b-00b5-4bb3-8e95-56d3f933e6a4"),
                created_at: gen_datetime("2024-11-02T00:00:00.000"),
                tenant: "foo".to_owned(),
                name: "Test bar key".to_string(),
                key: "a3dd31a59c493fcbb87c1b7acfa1770740de6a712e11337648f42d64420ff4bc".to_string(),
                masked: "bar-k************-key".to_string(),
                last_used: None,
                last_used_monitor_id: None,
                last_used_monitor_name: None,
            },
            ApiKeyData {
                api_key_id: gen_uuid("ea137deb-dfe0-4dca-bfd4-019492a522b1"),
                created_at: gen_datetime("2024-11-03T00:00:00.000"),
                tenant: "bar".to_owned(),
                name: "Test baz key".to_string(),
                key: "03c8d72da14dd44e7a1310dc396a4c36d9bb4cd941500b599285a55803070bb8".to_string(),
                masked: "baz-k************-key".to_string(),
                last_used: None,
                last_used_monitor_id: None,
                last_used_monitor_name: None,
            },
        ],
    )
}

pub async fn seed_db(
    monitor_seeds: &Vec<MonitorData>,
    job_seeds: &Vec<JobData>,
    api_key_seeds: &Vec<ApiKeyData>,
) -> DbPool {
    let pool = create_connection_pool().expect("Failed to setup DB connection pool");

    let mut conn = pool
        .get()
        .await
        .expect("Failed to retrieve DB connection from the pool");

    diesel::delete(monitor::table)
        .execute(&mut conn)
        .await
        .expect("Failed to delete existing monitor data");

    diesel::delete(api_key::table)
        .execute(&mut conn)
        .await
        .expect("Failed to delete existing api_key data");

    diesel::insert_into(monitor::table)
        .values(monitor_seeds)
        .execute(&mut conn)
        .await
        .expect("Failed to seed monitors");

    diesel::insert_into(job::table)
        .values(job_seeds)
        .execute(&mut conn)
        .await
        .expect("Failed to seed jobs");

    diesel::insert_into(api_key::table)
        .values(api_key_seeds)
        .execute(&mut conn)
        .await
        .expect("Failed to seed api_keys");

    pool
}

pub fn create_auth_header<'a>(kid: &str, name: &str, tenant: &str) -> Header<'a> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    Header::new(
        "Authorization",
        format!(
            "Bearer {}",
            encode_jwt(
                kid,
                &Jwt {
                    acr: "acr".to_string(),
                    azp: "azp".to_string(),
                    iss: "iss".to_string(),
                    jti: "jti".to_string(),
                    iat: now,
                    auth_time: now,
                    exp: now + 3600,
                    sub: "test-user".to_string(),
                    name: name.to_string(),
                    tenant: tenant.to_string(),
                }
            )
        ),
    )
}

async fn setup_mock_jwks_server(kid: &str) -> MockServer {
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/certs"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "keys": [
                {
                    "kid": kid.to_string(),
                    "kty": "RSA".to_string(),
                    "alg": "RS256".to_string(),
                    "n": RSA_MODULUS.to_string(),
                    "e": RSA_EXPONENT.to_string(),
                }
            ]
        })))
        .mount(&mock_server)
        .await;

    std::env::set_var("KEYCLOAK_CERTS_URL", format!("{}/certs", mock_server.uri()));

    mock_server
}
