use std::collections::HashMap;

use async_trait::async_trait;
use diesel::prelude::*;
use diesel::result::Error as DieselError;
use diesel_async::AsyncConnection;
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use uuid::Uuid;

use crate::errors::Error;
use crate::infrastructure::db_schema::api_key;
use crate::infrastructure::models::api_key::ApiKeyData;
use crate::infrastructure::repositories::api_keys::GetByKey;
use crate::infrastructure::repositories::Repository;

pub struct ApiKeyRepository<'a> {
    db: &'a mut AsyncPgConnection,
    data: HashMap<Uuid, ApiKeyData>,
}

impl<'a> ApiKeyRepository<'a> {
    pub fn new(db: &'a mut AsyncPgConnection) -> Self {
        Self {
            db,
            data: HashMap::new(),
        }
    }

    fn record_key(&mut self, key: ApiKeyData) -> ApiKeyData {
        self.data.insert(key.api_key_id, key.clone());
        key
    }
}

#[async_trait]
impl<'a> GetByKey for ApiKeyRepository<'a> {
    async fn get_by_key(&mut self, key: &str) -> Result<ApiKeyData, Error> {
        let result = self
            .db
            .transaction::<ApiKeyData, DieselError, _>(|conn| {
                Box::pin(async move {
                    let api_key = api_key::table
                        .filter(api_key::key.eq(key))
                        .select(ApiKeyData::as_select())
                        .first(conn)
                        .await?;

                    Ok(api_key)
                })
            })
            .await;

        match result {
            Err(e) => Err(Error::RepositoryError(e.to_string())),
            Ok(api_key) => Ok(self.record_key(api_key)),
        }
    }
}

#[async_trait]
impl<'a> Repository<ApiKeyData> for ApiKeyRepository<'a> {
    async fn get(
        &mut self,
        api_key_id: Uuid,
        tenant: Option<String>,
    ) -> Result<Option<ApiKeyData>, Error> {
        let result = self
            .db
            .transaction::<Option<ApiKeyData>, DieselError, _>(|conn| {
                Box::pin(async move {
                    let mut query = api_key::table
                        .select(ApiKeyData::as_select())
                        .filter(api_key::api_key_id.eq(api_key_id))
                        .into_boxed();
                    if let Some(tenant) = tenant {
                        query = query.filter(api_key::tenant.eq(tenant));
                    }

                    let api_key = query.first(conn).await.optional()?;

                    Ok(if let Some(key) = api_key {
                        Some(key)
                    } else {
                        None
                    })
                })
            })
            .await;

        match result {
            Err(e) => Err(Error::RepositoryError(e.to_string())),
            Ok(None) => Ok(None),
            Ok(Some(key)) => Ok(Some(self.record_key(key))),
        }
    }

    async fn all(&mut self, tenant: Option<String>) -> Result<Vec<ApiKeyData>, Error> {
        let result = self
            .db
            .transaction::<Vec<ApiKeyData>, DieselError, _>(|conn| {
                Box::pin(async move {
                    let mut query = api_key::table.select(ApiKeyData::as_select()).into_boxed();
                    if let Some(tenant) = tenant {
                        query = query.filter(api_key::tenant.eq(tenant));
                    }

                    let api_keys = query.load(conn).await?;

                    Ok(api_keys)
                })
            })
            .await;

        match result {
            Err(e) => Err(Error::RepositoryError(e.to_string())),
            Ok(keys) => Ok(keys.into_iter().map(|k| self.record_key(k)).collect()),
        }
    }

    async fn save(&mut self, api_key: &ApiKeyData) -> Result<(), Error> {
        let cached_data = self.data.get(&api_key.api_key_id);

        let result = self
            .db
            .transaction::<ApiKeyData, DieselError, _>(|conn| {
                Box::pin(async move {
                    if let Some(_cached) = cached_data {
                        diesel::update(api_key).set(api_key).execute(conn).await?;
                    } else {
                        diesel::insert_into(api_key::table)
                            .values(api_key)
                            .execute(conn)
                            .await?;
                    }

                    Ok(api_key.clone())
                })
            })
            .await;

        match result {
            Err(e) => Err(Error::RepositoryError(e.to_string())),
            Ok(key) => {
                self.record_key(key);
                Ok(())
            }
        }
    }

    async fn delete(&mut self, key: &ApiKeyData) -> Result<(), Error> {
        let result = diesel::delete(&key).execute(self.db).await;

        match result {
            Err(e) => Err(Error::RepositoryError(e.to_string())),
            Ok(_) => {
                self.data.remove(&key.api_key_id);
                Ok(())
            }
        }
    }
}
