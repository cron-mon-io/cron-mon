use std::collections::HashMap;

use async_trait::async_trait;
use diesel::prelude::*;
use diesel::result::Error as DieselError;
use diesel_async::AsyncConnection;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::domain::models::api_key::ApiKey;
use crate::errors::Error;
use crate::infrastructure::database::{get_connection, DbPool};
use crate::infrastructure::db_schema::api_key;
use crate::infrastructure::models::api_key::ApiKeyData;
use crate::infrastructure::repositories::api_keys::GetByKey;
use crate::infrastructure::repositories::Repository;

pub struct ApiKeyRepository<'a> {
    pool: &'a DbPool,
    data: HashMap<Uuid, ApiKeyData>,
}

impl<'a> ApiKeyRepository<'a> {
    pub fn new(pool: &'a DbPool) -> Self {
        Self {
            pool,
            data: HashMap::new(),
        }
    }

    fn db_to_key(&mut self, key: &ApiKeyData) -> ApiKey {
        let api_key = ApiKey::from(key);
        self.data.insert(key.api_key_id, key.clone());
        api_key
    }
}

#[async_trait]
impl<'a> GetByKey for ApiKeyRepository<'a> {
    async fn get_by_key(&mut self, key: &str) -> Result<Option<ApiKey>, Error> {
        let mut connection = get_connection(self.pool).await?;
        connection
            .transaction::<Option<ApiKey>, DieselError, _>(|conn| {
                Box::pin(async move {
                    let api_key = api_key::table
                        .filter(api_key::key.eq(key))
                        .select(ApiKeyData::as_select())
                        .first(conn)
                        .await
                        .optional()?;

                    Ok(api_key.map(|key| self.db_to_key(&key)))
                })
            })
            .await
            .map_err(|err| Error::RepositoryError(err.to_string()))
    }
}

#[async_trait]
impl<'a> Repository<ApiKey> for ApiKeyRepository<'a> {
    async fn get(&mut self, api_key_id: Uuid, tenant: &str) -> Result<Option<ApiKey>, Error> {
        let mut connection = get_connection(self.pool).await?;
        connection
            .transaction::<Option<ApiKey>, DieselError, _>(|conn| {
                Box::pin(async move {
                    let api_key = api_key::table
                        .select(ApiKeyData::as_select())
                        .filter(
                            api_key::api_key_id
                                .eq(api_key_id)
                                .and(api_key::tenant.eq(tenant)),
                        )
                        .first(conn)
                        .await
                        .optional()?;

                    Ok(api_key.map(|key| self.db_to_key(&key)))
                })
            })
            .await
            .map_err(|err| Error::RepositoryError(err.to_string()))
    }

    async fn all(&mut self, tenant: &str) -> Result<Vec<ApiKey>, Error> {
        let mut connection = get_connection(self.pool).await?;
        connection
            .transaction::<Vec<ApiKey>, DieselError, _>(|conn| {
                Box::pin(async move {
                    let api_keys = api_key::table
                        .select(ApiKeyData::as_select())
                        .filter(api_key::tenant.eq(tenant))
                        .load(conn)
                        .await?;

                    Ok(api_keys.into_iter().map(|k| self.db_to_key(&k)).collect())
                })
            })
            .await
            .map_err(|err| Error::RepositoryError(err.to_string()))
    }

    async fn save(&mut self, key: &ApiKey) -> Result<(), Error> {
        let mut connection = get_connection(self.pool).await?;
        connection
            .transaction::<(), DieselError, _>(|conn| {
                Box::pin(async move {
                    let api_key_data = ApiKeyData::from(key);
                    if let Some(_cached) = self.data.get(&api_key_data.api_key_id) {
                        diesel::update(&api_key_data)
                            .set(&api_key_data)
                            .execute(conn)
                            .await?;
                    } else {
                        diesel::insert_into(api_key::table)
                            .values(&api_key_data)
                            .execute(conn)
                            .await?;
                    }

                    self.db_to_key(&api_key_data);

                    Ok(())
                })
            })
            .await
            .map_err(|err| Error::RepositoryError(err.to_string()))
    }

    async fn delete(&mut self, key: &ApiKey) -> Result<(), Error> {
        let api_key_data = ApiKeyData::from(key);
        let mut connection = get_connection(self.pool).await?;
        diesel::delete(&api_key_data)
            .execute(&mut connection)
            .await
            .map_err(|err| Error::RepositoryError(err.to_string()))?;

        self.data.remove(&key.api_key_id);
        Ok(())
    }
}
