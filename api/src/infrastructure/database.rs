use rocket_db_pools::{diesel, Database};

#[derive(Database)]
#[database("monitors")]
pub struct Db(diesel::PgPool);
