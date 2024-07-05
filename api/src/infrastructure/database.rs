use std::env;

use diesel_async::{AsyncConnection, AsyncPgConnection};
use rocket::futures::TryFutureExt;
use rocket_db_pools::{diesel, Database};

#[derive(Database)]
#[database("monitors")]
pub struct Db(diesel::PgPool);

pub async fn establish_connection() -> AsyncPgConnection {
    let database_url = env::var("DATABASE_URL").expect("'DATABASE_URL' missing from environment");
    AsyncPgConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Failed to establish DB connection"))
        .await
}
