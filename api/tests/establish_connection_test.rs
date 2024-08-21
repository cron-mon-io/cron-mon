use std::env;

use tokio::test;

use cron_mon_api::{errors::Error, infrastructure::database::establish_connection};

#[test]
async fn test_establish_connection() {
    let conn_result = establish_connection().await;
    assert!(conn_result.is_ok());
}

#[test]
async fn test_establish_connection_error() {
    env::set_var(
        "DATABASE_URL",
        "postgres://postgres:password@localhost:5432/monitors_test",
    );

    let conn = establish_connection().await;
    assert!(conn.is_err());
    assert!(matches!(conn.err().unwrap(), Error::RepositoryError { .. }));
}
