use std::collections::HashMap;

use async_trait::async_trait;
use diesel::prelude::*;
use diesel::result::Error as DieselError;
use diesel_async::AsyncConnection;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::domain::models::alert_config::AlertConfig;
use crate::errors::Error;
use crate::infrastructure::database::{get_connection, DbPool};
use crate::infrastructure::db_schema::{alert_config, slack_alert_config};
use crate::infrastructure::models::alert_config::AlertConfigReadData;
use crate::infrastructure::repositories::Repository;

macro_rules! build_polymorphic_query {
    () => {{
        alert_config::table
            .left_join(
                slack_alert_config::dsl::slack_alert_config
                    .on(slack_alert_config::dsl::alert_config_id.eq(alert_config::alert_config_id)),
            )
            .select((
                alert_config::alert_config_id,
                alert_config::name,
                alert_config::tenant,
                alert_config::type_,
                alert_config::active,
                alert_config::on_late,
                alert_config::on_error,
                slack_alert_config::dsl::slack_channel.nullable(),
                slack_alert_config::dsl::slack_bot_oauth_token.nullable(),
            ))
            .into_boxed()
    }};
}

pub struct AlertConfigRepository<'a> {
    pool: &'a DbPool,
    data: HashMap<Uuid, AlertConfigReadData>,
}

impl<'a> AlertConfigRepository<'a> {
    pub fn new(pool: &'a DbPool) -> Self {
        Self {
            pool,
            data: HashMap::new(),
        }
    }

    fn db_to_key(&mut self, alert_config_data: &AlertConfigReadData) -> Result<AlertConfig, Error> {
        let alert_config = alert_config_data.to_model()?;
        self.data
            .insert(alert_config.alert_config_id, alert_config_data.clone());
        Ok(alert_config)
    }
}

#[async_trait]
impl<'a> Repository<AlertConfig> for AlertConfigRepository<'a> {
    async fn get(
        &mut self,
        alert_config_id: Uuid,
        tenant: &str,
    ) -> Result<Option<AlertConfig>, Error> {
        let mut connection = get_connection(self.pool).await?;
        let result = connection
            .transaction::<Option<AlertConfigReadData>, DieselError, _>(|conn| {
                Box::pin(async move {
                    build_polymorphic_query!()
                        .filter(
                            alert_config::alert_config_id
                                .eq(alert_config_id)
                                .and(alert_config::tenant.eq(tenant)),
                        )
                        .first(conn)
                        .await
                        .optional()
                })
            })
            .await
            .map_err(|err| Error::RepositoryError(err.to_string()))?;

        Ok(match result {
            None => None,
            Some(alert_config_data) => Some(self.db_to_key(&alert_config_data)?),
        })
    }

    async fn all(&mut self, tenant: &str) -> Result<Vec<AlertConfig>, Error> {
        let mut connection = get_connection(self.pool).await?;
        let results = connection
            .transaction::<Vec<AlertConfigReadData>, DieselError, _>(|conn| {
                Box::pin(async move {
                    build_polymorphic_query!()
                        .filter(alert_config::tenant.eq(tenant))
                        .load(conn)
                        .await
                })
            })
            .await
            .map_err(|err| Error::RepositoryError(err.to_string()))?;

        Ok(results
            .iter()
            .map(|data| self.db_to_key(data))
            .collect::<Result<Vec<AlertConfig>, Error>>()?)
    }

    async fn save(&mut self, _alert_config: &AlertConfig) -> Result<(), Error> {
        todo!()
    }

    async fn delete(&mut self, _alert_config: &AlertConfig) -> Result<(), Error> {
        todo!()
    }
}
