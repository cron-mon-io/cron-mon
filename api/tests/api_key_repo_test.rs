pub mod common;

use pretty_assertions::assert_eq;
use rstest::rstest;

use test_utils::gen_uuid;

use cron_mon_api::infrastructure::models::api_key::ApiKeyData;
use cron_mon_api::infrastructure::repositories::api_key_repo::ApiKeyRepository;
use cron_mon_api::infrastructure::repositories::api_keys::GetByKey;
use cron_mon_api::infrastructure::repositories::Repository;

use common::setup_db;

#[rstest]
#[case(Some("foo".to_owned()), vec!["foo-key", "bar-key"])]
#[case(None, vec!["foo-key", "bar-key", "baz-key"])]
#[tokio::test]
async fn test_all(#[case] tenant: Option<String>, #[case] expected_keys: Vec<&str>) {
    // See data seeds for the expected data (/api/tests/common/mod.rs)
    let mut conn = setup_db().await;
    let mut repo = ApiKeyRepository::new(&mut conn);

    let keys = repo.all(tenant).await.unwrap();

    let keys: Vec<String> = keys.iter().map(|key| key.key.clone()).collect();
    assert_eq!(keys, expected_keys);
}

#[tokio::test]
async fn test_get() {
    let mut conn = setup_db().await;
    let mut repo = ApiKeyRepository::new(&mut conn);

    let non_existent_api_key_id = repo
        .get(
            gen_uuid("4940ede2-72fc-4e0e-838e-f15f35e3594f"),
            Some("foo".to_owned()),
        )
        .await
        .unwrap();
    let wrong_tenant = repo
        .get(
            gen_uuid("bfab6d41-8b00-49ef-86df-f562b701ee4f"),
            Some("bar".to_owned()),
        )
        .await
        .unwrap();
    let should_be_some_with_tenant = repo
        .get(
            gen_uuid("bfab6d41-8b00-49ef-86df-f562b701ee4f"),
            Some("foo".to_owned()),
        )
        .await
        .unwrap();
    let should_be_some_without_tenant = repo
        .get(gen_uuid("bfab6d41-8b00-49ef-86df-f562b701ee4f"), None)
        .await
        .unwrap();

    assert!(non_existent_api_key_id.is_none());
    assert!(wrong_tenant.is_none());
    assert!(should_be_some_with_tenant.is_some());
    assert!(should_be_some_without_tenant.is_some());

    let key_with_tenant = should_be_some_with_tenant.unwrap();
    assert_eq!(key_with_tenant.key, "foo-key");

    let key_without_tenant = should_be_some_without_tenant.unwrap();
    assert_eq!(key_without_tenant.key, "foo-key");
}

#[tokio::test]
async fn test_get_by_key() {
    let mut conn = setup_db().await;
    let mut repo = ApiKeyRepository::new(&mut conn);

    let key = repo.get_by_key("foo-key").await.unwrap();
    assert_eq!(
        key.api_key_id,
        gen_uuid("bfab6d41-8b00-49ef-86df-f562b701ee4f")
    );
}

#[tokio::test]
async fn test_save() {
    let mut conn = setup_db().await;
    let mut repo = ApiKeyRepository::new(&mut conn);

    let new_api_key = ApiKeyData {
        api_key_id: gen_uuid("d2b291fe-bd41-4787-bc2d-1329903f7a6a"),
        tenant: "foo".to_string(),
        key: "new-key".to_string(),
        last_used: None,
        last_used_monitor_id: None,
        last_used_monitor_name: None,
    };
    repo.save(&new_api_key).await.unwrap();
    assert_eq!(repo.all(Some("foo".to_owned())).await.unwrap().len(), 3);

    let read_new_api_key = repo
        .get(new_api_key.api_key_id, Some("foo".to_owned()))
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
    let mut conn = setup_db().await;
    let mut repo = ApiKeyRepository::new(&mut conn);

    let key = repo
        .get(
            gen_uuid("bfab6d41-8b00-49ef-86df-f562b701ee4f"),
            Some("foo".to_owned()),
        )
        .await
        .unwrap()
        .unwrap();

    repo.delete(&key).await.unwrap();
    assert!(repo.get(key.api_key_id, None).await.unwrap().is_none());
    assert_eq!(repo.all(Some("foo".to_owned())).await.unwrap().len(), 1);
}
