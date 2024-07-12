use std::env;

use diesel::Connection;
use diesel::PgConnection;
use diesel_async::{AsyncConnection, AsyncPgConnection};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use rocket_db_pools::{diesel, Database};

use crate::errors::AppError;

#[derive(Database)]
#[database("monitors")]
pub struct Db(diesel::PgPool);

const MIGRATIONS: EmbeddedMigrations = embed_migrations!("src/infrastructure/migrations");

pub async fn establish_connection() -> Result<AsyncPgConnection, AppError> {
    match AsyncPgConnection::establish(&get_database_url()).await {
        Ok(conn) => Ok(conn),
        Err(e) => Err(AppError::RepositoryError(format!(
            "Failed to establish DB connection: {:?}",
            e
        ))),
    }
}

pub fn run_migrations() {
    let mut conn = PgConnection::establish(&get_database_url())
        .unwrap_or_else(|_| panic!("Failed to establish DB connection"));

    println!("Running migrations...");
    conn.run_pending_migrations(MIGRATIONS)
        .unwrap_or_else(|_| panic!("Failed to run migrations"));
    println!("Migrations complete");
}

fn get_database_url() -> String {
    env::var("DATABASE_URL").expect("'DATABASE_URL' missing from environment")
}
