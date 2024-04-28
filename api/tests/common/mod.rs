use std::fs::read_to_string;

use diesel::dsl::sql_query;
use diesel_async::AsyncPgConnection;
use diesel_async::RunQueryDsl;
use rocket::local::blocking::Client;

use cron_mon_api::infrastructure::database::establish_connection;
use cron_mon_api::rocket;

pub async fn setup_db() -> AsyncPgConnection {
    let seed_queries = get_seed_queries();

    let mut conn = establish_connection().await;
    for query in seed_queries {
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

fn get_seed_queries() -> Vec<String> {
    let seed_script = read_to_string("src/infrastructure/seeding/seeds.sql")
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
