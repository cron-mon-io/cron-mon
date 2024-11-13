pub mod common;

use rstest::rstest;

use cron_mon_api::infrastructure::database::create_connection_pool;

use common::{infrastructure, Infrastructure};

#[rstest]
#[tokio::test]
async fn test_create_connection_pool(#[future] infrastructure: Infrastructure) {
    let _infra = infrastructure.await;

    let pool_result = create_connection_pool();
    assert!(pool_result.is_ok());

    let pool = pool_result.unwrap();

    let conn_result = pool.get().await;
    assert!(conn_result.is_ok());
}

#[tokio::test]
async fn test_create_connection_pool_error() {
    let conn_result = create_connection_pool().unwrap().get().await;
    assert!(conn_result.is_err());
}
