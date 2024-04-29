use std::env;
use std::fs::read_to_string;
use std::str::FromStr;

use chrono::NaiveDateTime;
use diesel::dsl::sql_query;
use diesel_async::AsyncPgConnection;
use diesel_async::RunQueryDsl;
use rocket::local::blocking::Client;
use uuid::Uuid;

use cron_mon_api::infrastructure::database::establish_connection;
use cron_mon_api::rocket;

pub async fn setup_db() -> AsyncPgConnection {
    let seed_queries = get_seed_queries();
    println!("Running {} seed queries...", seed_queries.len());

    let mut conn = establish_connection().await;
    for query in seed_queries {
        println!("RUNNING: {}", query);
        sql_query(query)
            .execute(&mut conn)
            .await
            .expect("Failed to seed database");
    }

    conn
}

pub fn get_test_client() -> Client {
    Client::tracked(rocket()).expect("Invalid rocket instance")
}

// Duplicate of src/infrastructure/respositories/test_repo - need to figure out a better way...
pub fn gen_uuid(uuid: &str) -> Uuid {
    Uuid::from_str(uuid).unwrap()
}

pub fn is_uuid(uuid: &str) -> bool {
    if let Ok(_) = Uuid::from_str(uuid) {
        true
    } else {
        false
    }
}

pub fn is_datetime(datetime: &str) -> bool {
    if let Ok(_) = NaiveDateTime::parse_from_str(datetime, "%Y-%m-%dT%H:%M:%S%.f") {
        true
    } else {
        false
    }
}

fn get_seed_queries() -> Vec<String> {
    let seed_script = read_to_string(env::var("SEED_SCRIPT_PATH").unwrap())
        .unwrap()
        .lines()
        .filter_map(|line| {
            // Filter out blank lines and comments.
            if line.contains("--") || line.is_empty() {
                None
            } else {
                Some(line)
            }
        })
        .collect::<Vec<&str>>()
        .join("");

    seed_script
        .split(";")
        .map(|script| script.to_owned())
        .collect::<Vec<String>>()
}
