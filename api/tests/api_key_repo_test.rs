pub mod common;

use pretty_assertions::assert_eq;
use rstest::rstest;

use test_utils::gen_uuid;

use cron_mon_api::domain::models::ApiKey;
use cron_mon_api::infrastructure::repositories::api_key::{ApiKeyRepository, GetByKey};
use cron_mon_api::infrastructure::repositories::Repository;

use common::{infrastructure, Infrastructure};

#[rstest]
#[tokio::test]
async fn test_all(#[future] infrastructure: Infrastructure) {
    let infra = infrastructure.await;
    let mut repo = ApiKeyRepository::new(&infra.pool);

    let keys = repo.all("foo").await.unwrap();

    let keys: Vec<String> = keys.iter().map(|key| key.key.clone()).collect();
    assert_eq!(
        keys,
        vec![
            "104e4587f5340bd9264ea0fee2075627c74420bd5c48aa9e8a463f03a2675020",
            "a3dd31a59c493fcbb87c1b7acfa1770740de6a712e11337648f42d64420ff4bc"
        ]
    );
}

#[rstest]
#[tokio::test]
async fn test_get(#[future] infrastructure: Infrastructure) {
    let infra = infrastructure.await;
    let mut repo = ApiKeyRepository::new(&infra.pool);

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
    assert_eq!(
        key.key,
        "104e4587f5340bd9264ea0fee2075627c74420bd5c48aa9e8a463f03a2675020"
    );
}

#[rstest]
#[tokio::test]
async fn test_get_by_key(#[future] infrastructure: Infrastructure) {
    let infra = infrastructure.await;
    let mut repo = ApiKeyRepository::new(&infra.pool);

    let existent_key = repo
        .get_by_key("104e4587f5340bd9264ea0fee2075627c74420bd5c48aa9e8a463f03a2675020")
        .await
        .unwrap();
    let non_existent_key = repo.get_by_key("non-existent").await.unwrap();

    assert!(existent_key.is_some());
    assert_eq!(
        existent_key.unwrap().api_key_id,
        gen_uuid("bfab6d41-8b00-49ef-86df-f562b701ee4f")
    );

    assert!(non_existent_key.is_none());
}

#[rstest]
#[tokio::test]
async fn test_save(#[future] infrastructure: Infrastructure) {
    let infra = infrastructure.await;
    let mut repo = ApiKeyRepository::new(&infra.pool);

    let new_api_key = ApiKey::new(
        "New key".to_string(),
        "new-key".to_string(),
        "foo".to_string(),
    );
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

#[rstest]
#[tokio::test]
async fn test_delete(#[future] infrastructure: Infrastructure) {
    let infra = infrastructure.await;
    let mut repo = ApiKeyRepository::new(&infra.pool);

    let key = repo
        .get(gen_uuid("bfab6d41-8b00-49ef-86df-f562b701ee4f"), "foo")
        .await
        .unwrap()
        .unwrap();

    repo.delete(&key).await.unwrap();
    assert!(repo.get(key.api_key_id, "foo").await.unwrap().is_none());
    assert_eq!(repo.all("foo").await.unwrap().len(), 1);
}
