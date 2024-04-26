use std::fs::read_to_string;

use diesel::dsl::sql_query;
use diesel_async::AsyncPgConnection;

use cron_mon_api::infrastructure::database::establish_connection;
use diesel_async::RunQueryDsl;

pub async fn setup_db() -> AsyncPgConnection {
    let seed_script = read_to_string("src/infrastructure/seeding/seeds.sql")
        .unwrap()
        .lines()
        .filter_map(|line| {
            if line.contains("--") || line.is_empty() {
                None
            } else {
                Some(line)
            }
        })
        .collect::<Vec<&str>>()
        .join("");
    let seed_queries: Vec<&str> = seed_script.split(";").collect();

    let mut conn = establish_connection().await;
    for query in seed_queries {
        sql_query(query)
            .execute(&mut conn)
            .await
            .expect("Failed to seed database");
    }

    conn
}
