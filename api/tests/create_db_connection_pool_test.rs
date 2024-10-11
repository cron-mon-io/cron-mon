use std::env;

use cron_mon_api::infrastructure::database::create_db_connection_pool;

#[tokio::test]
async fn test_create_db_connection_pool() {
    let pool_result = create_db_connection_pool();
    assert!(pool_result.is_ok());

    let pool = pool_result.unwrap();

    let conn_result = pool.get().await;
    assert!(conn_result.is_ok());
}

#[tokio::test]
async fn test_create_db_connection_pool_error() {
    env::set_var(
        "DATABASE_URL",
        "postgres://postgres:password@localhost:5432/monitors_test",
    );

    let conn_result = create_db_connection_pool().unwrap().get().await;
    assert!(conn_result.is_err());
}
