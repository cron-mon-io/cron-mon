pub mod common;

use pretty_assertions::assert_eq;

use test_utils::gen_uuid;

use cron_mon_api::domain::models::api_key::ApiKey;
use cron_mon_api::infrastructure::repositories::api_key_repo::ApiKeyRepository;
use cron_mon_api::infrastructure::repositories::api_keys::GetByKey;
use cron_mon_api::infrastructure::repositories::Repository;

use common::setup_db_pool;

#[tokio::test]
async fn test_all() {
    // See data seeds for the expected data (/api/tests/common/mod.rs)
    let pool = setup_db_pool().await;
    let mut repo = ApiKeyRepository::new(&pool);

    let keys = repo.all("foo").await.unwrap();

    let keys: Vec<String> = keys.iter().map(|key| key.key.clone()).collect();
    assert_eq!(keys, vec!["foo-key", "bar-key"]);
}

#[tokio::test]
async fn test_get() {
    let pool = setup_db_pool().await;
    let mut repo = ApiKeyRepository::new(&pool);

    let non_existent_api_key_id = repo
        .get(gen_uuid("4940ede2-72fc-4e0e-838e-f15f35e3594f"), "foo")
        .await
        .unwrap();
    let wrong_tenant = repo
        .get(gen_uuid("bfab6d41-8b00-49ef-86df-f562b701ee4f"), "bar")
        .await
        .unwrap();
    let should_be_some = repo
        .get(gen_uuid("bfab6d41-8b00-49ef-86df-f562b701ee4f"), "foo")
        .await
        .unwrap();

    assert!(non_existent_api_key_id.is_none());
    assert!(wrong_tenant.is_none());
    assert!(should_be_some.is_some());

    let key = should_be_some.unwrap();
    assert_eq!(key.key, "foo-key");
}

#[tokio::test]
async fn test_get_by_key() {
    let pool = setup_db_pool().await;
    let mut repo = ApiKeyRepository::new(&pool);

    let existent_key = repo.get_by_key("foo-key").await.unwrap();
    let non_existent_key = repo.get_by_key("non-existent").await.unwrap();

    assert!(existent_key.is_some());
    assert_eq!(
        existent_key.unwrap().api_key_id,
        gen_uuid("bfab6d41-8b00-49ef-86df-f562b701ee4f")
    );

    assert!(non_existent_key.is_none());
}

#[tokio::test]
async fn test_save() {
    let pool = setup_db_pool().await;
    let mut repo = ApiKeyRepository::new(&pool);

    let new_api_key = ApiKey {
        api_key_id: gen_uuid("d2b291fe-bd41-4787-bc2d-1329903f7a6a"),
        tenant: "foo".to_string(),
        key: "new-key".to_string(),
        last_used: None,
        last_used_monitor_id: None,
        last_used_monitor_name: None,
    };
    repo.save(&new_api_key).await.unwrap();
    assert_eq!(repo.all("foo").await.unwrap().len(), 3);

    let read_new_api_key = repo
        .get(new_api_key.api_key_id, "foo")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(new_api_key.api_key_id, read_new_api_key.api_key_id);
    assert_eq!(new_api_key.key, read_new_api_key.key);
    assert_eq!(new_api_key.last_used, read_new_api_key.last_used);
    assert_eq!(
        new_api_key.last_used_monitor_id,
        read_new_api_key.last_used_monitor_id
    );
    assert_eq!(
        new_api_key.last_used_monitor_name,
        read_new_api_key.last_used_monitor_name
    );
}

#[tokio::test]
async fn test_delete() {
    let pool = setup_db_pool().await;
    let mut repo = ApiKeyRepository::new(&pool);

    let key = repo
        .get(gen_uuid("bfab6d41-8b00-49ef-86df-f562b701ee4f"), "foo")
        .await
        .unwrap()
        .unwrap();

    repo.delete(&key).await.unwrap();
    assert!(repo.get(key.api_key_id, "foo").await.unwrap().is_none());
    assert_eq!(repo.all("foo").await.unwrap().len(), 1);
}
