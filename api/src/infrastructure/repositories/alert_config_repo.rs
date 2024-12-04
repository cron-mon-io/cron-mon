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
use crate::infrastructure::models::alert_config::AlertConfigData;
use crate::infrastructure::models::alert_config::NewAlertConfigData;
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
    data: HashMap<Uuid, AlertConfig>,
}

impl<'a> AlertConfigRepository<'a> {
    pub fn new(pool: &'a DbPool) -> Self {
        Self {
            pool,
            data: HashMap::new(),
        }
    }

    fn db_to_model(&mut self, alert_config_data: &AlertConfigData) -> Result<AlertConfig, Error> {
        let alert_config = alert_config_data.to_model()?;
        self.data
            .insert(alert_config.alert_config_id, alert_config.clone());
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
            .transaction::<Option<AlertConfigData>, DieselError, _>(|conn| {
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
            Some(alert_config_data) => Some(self.db_to_model(&alert_config_data)?),
        })
    }

    async fn all(&mut self, tenant: &str) -> Result<Vec<AlertConfig>, Error> {
        let mut connection = get_connection(self.pool).await?;
        let results = connection
            .transaction::<Vec<AlertConfigData>, DieselError, _>(|conn| {
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
            .map(|data| self.db_to_model(data))
            .collect::<Result<Vec<AlertConfig>, Error>>()?)
    }

    async fn save(&mut self, alert_config: &AlertConfig) -> Result<(), Error> {
        let (alert_config_data, slack_alert_config_data) =
            NewAlertConfigData::from_model(alert_config);

        // We can do this now as we only support Slack, but when we add more integrations we will
        // need to handle this differently.
        let slack_alert_config_data = slack_alert_config_data.unwrap();

        let mut connection = get_connection(self.pool).await?;
        connection
            .transaction::<(), DieselError, _>(|conn| {
                Box::pin(async {
                    if self.data.contains_key(&alert_config.alert_config_id) {
                        diesel::update(&alert_config_data)
                            .set(&alert_config_data)
                            .execute(conn)
                            .await?;
                        diesel::update(&slack_alert_config_data)
                            .set(&slack_alert_config_data)
                            .execute(conn)
                            .await?;
                    } else {
                        diesel::insert_into(alert_config::table)
                            .values(&alert_config_data)
                            .execute(conn)
                            .await?;

                        diesel::insert_into(slack_alert_config::table)
                            .values(&slack_alert_config_data)
                            .execute(conn)
                            .await?;
                    }

                    self.data
                        .insert(alert_config.alert_config_id, alert_config.clone());
                    Ok(())
                })
            })
            .await
            .map_err(|err| Error::RepositoryError(err.to_string()))
    }

    async fn delete(&mut self, alert_config: &AlertConfig) -> Result<(), Error> {
        let mut connection = get_connection(self.pool).await?;

        // We only need to delete the alert_config row, as the foreign key constraint will take care
        // of deleting the integration specific rows via `ON DELETE CASCADE`.
        diesel::delete(
            alert_config::table
                .filter(alert_config::alert_config_id.eq(alert_config.alert_config_id)),
        )
        .execute(&mut connection)
        .await
        .map_err(|err| Error::RepositoryError(err.to_string()))?;

        self.data.remove(&alert_config.alert_config_id);
        Ok(())
    }
}
