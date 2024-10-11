use std::env;

use diesel::Connection;
use diesel::PgConnection;
use diesel_async::pooled_connection::{deadpool::Pool, AsyncDieselConnectionManager};
use diesel_async::{AsyncConnection, AsyncPgConnection};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

use crate::errors::Error;

const MIGRATIONS: EmbeddedMigrations = embed_migrations!("src/infrastructure/migrations");

pub type DbPool = Pool<AsyncPgConnection>;

pub fn create_db_connection_pool() -> Result<DbPool, Error> {
    let manager = AsyncDieselConnectionManager::<AsyncPgConnection>::new(get_database_url());

    let pool = Pool::builder(manager)
        .build()
        .expect("Failed to create DB connection pool.");

    Ok(pool)
}

// TODO: Bin this off and just use create_db_connection_pool.
pub async fn establish_connection() -> Result<AsyncPgConnection, Error> {
    match AsyncPgConnection::establish(&get_database_url()).await {
        Ok(conn) => Ok(conn),
        Err(e) => Err(Error::RepositoryError(format!(
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
