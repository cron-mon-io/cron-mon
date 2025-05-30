use std::collections::HashMap;

use async_trait::async_trait;
use diesel::prelude::*;
use diesel::result::Error as DieselError;
use diesel_async::pooled_connection::deadpool::Object;
use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl};
use uuid::Uuid;

use crate::domain::models::AlertConfig;
use crate::errors::Error;
use crate::infrastructure::database::{get_connection, DbPool};
use crate::infrastructure::db_schema::{alert_config, monitor_alert_config, slack_alert_config};
use crate::infrastructure::models::alert_config::NewSlackAlertConfigData;
use crate::infrastructure::models::alert_config::{
    AlertConfigData, MonitorAlertConfigData, NewAlertConfigData,
};
use crate::infrastructure::repositories::Repository;

use super::{GetByIDs, GetByMonitors};

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
            .distinct()
            .into_boxed()
    }};
}

enum FilterableIds<'a> {
    AlertConfigIds(&'a [Uuid]),
    MonitorIds(&'a [Uuid]),
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

    fn db_to_model(
        &mut self,
        alert_config_data: &AlertConfigData,
        monitor_alert_configs: &[MonitorAlertConfigData],
    ) -> Result<AlertConfig, Error> {
        let alert_config = alert_config_data.to_model(monitor_alert_configs)?;
        self.data
            .insert(alert_config.alert_config_id, alert_config.clone());
        Ok(alert_config)
    }

    async fn update(
        &mut self,
        conn: &mut Object<AsyncPgConnection>,
        alert_config_data: &NewAlertConfigData,
        slack_alert_config_data: &NewSlackAlertConfigData,
        monitor_alert_configs: &[MonitorAlertConfigData],
    ) -> Result<(), DieselError> {
        diesel::update(alert_config_data)
            .set(alert_config_data)
            .execute(conn)
            .await?;
        diesel::update(slack_alert_config_data)
            .set(slack_alert_config_data)
            .execute(conn)
            .await?;

        // Delete all monitor_alert_configs for the alert_config and insert the new ones. This is
        // inefficient in some scenarios, like when the list of monitors an alert is configured for
        // hasn't changed, since we'll be doing a DELETE and an INSERT unneccessarily, but it's the
        // simplest way to handle this for now, and it's not expected to be a common operation, or
        // that there'll be a large amount of data in either tables.
        diesel::delete(
            monitor_alert_config::table.filter(
                monitor_alert_config::alert_config_id.eq(alert_config_data.alert_config_id),
            ),
        )
        .execute(conn)
        .await?;

        diesel::insert_into(monitor_alert_config::table)
            .values(monitor_alert_configs)
            .execute(conn)
            .await?;

        Ok(())
    }

    async fn insert(
        &mut self,
        conn: &mut Object<AsyncPgConnection>,
        alert_config_data: &NewAlertConfigData,
        slack_alert_config_data: &NewSlackAlertConfigData,
        monitor_alert_configs: &[MonitorAlertConfigData],
    ) -> Result<(), DieselError> {
        diesel::insert_into(alert_config::table)
            .values(alert_config_data)
            .execute(conn)
            .await?;

        diesel::insert_into(slack_alert_config::table)
            .values(slack_alert_config_data)
            .execute(conn)
            .await?;

        diesel::insert_into(monitor_alert_config::table)
            .values(monitor_alert_configs)
            .execute(conn)
            .await?;

        Ok(())
    }

    async fn fetch_alert_configs(
        &mut self,
        tenant: Option<&str>,
        filterable_ids: Option<FilterableIds<'_>>,
    ) -> Result<Vec<AlertConfig>, Error> {
        let mut connection = get_connection(self.pool).await?;
        let (alert_config_datas, monitor_alert_config_datas) = connection
            .transaction::<(Vec<AlertConfigData>, Vec<MonitorAlertConfigData>), DieselError, _>(
                |conn| {
                    Box::pin(async move {
                        let mut query = build_polymorphic_query!();
                        if let Some(t) = tenant {
                            query = query.filter(alert_config::tenant.eq(t));
                        }
                        let alert_configs: Vec<AlertConfigData> = if let Some(filterable_ids) =
                            filterable_ids
                        {
                            match filterable_ids {
                                FilterableIds::AlertConfigIds(ids) => {
                                    query
                                        .filter(alert_config::alert_config_id.eq_any(ids))
                                        .load(conn)
                                        .await?
                                }
                                FilterableIds::MonitorIds(monitor_ids) => {
                                    query
                                        .inner_join(
                                            monitor_alert_config::table
                                                .on(monitor_alert_config::alert_config_id
                                                    .eq(alert_config::alert_config_id)),
                                        )
                                        .filter(
                                            monitor_alert_config::monitor_id.eq_any(monitor_ids),
                                        )
                                        .load(conn)
                                        .await?
                                }
                            }
                        } else {
                            query.load(conn).await?
                        };

                        let monitor_alert_configs =
                            MonitorAlertConfigData::belonging_to(&alert_configs)
                                .select(MonitorAlertConfigData::as_select())
                                .load(conn)
                                .await?;

                        Ok((alert_configs, monitor_alert_configs))
                    })
                },
            )
            .await
            .map_err(|err| Error::RepositoryError(err.to_string()))?;

        monitor_alert_config_datas
            .grouped_by(&alert_config_datas)
            .into_iter()
            .zip(alert_config_datas)
            .map(|(monitor_alert_config_datas, alert_config_datas)| {
                self.db_to_model(&alert_config_datas, &monitor_alert_config_datas)
            })
            .collect::<Result<Vec<AlertConfig>, Error>>()
    }
}

#[async_trait]
#[allow(clippy::needless_lifetimes)] // This is needed for the lifetime of the pool
impl<'a> GetByMonitors for AlertConfigRepository<'a> {
    async fn get_by_monitors<'b>(
        &mut self,
        monitor_ids: &[Uuid],
        tenant: Option<&'b str>,
    ) -> Result<Vec<AlertConfig>, Error> {
        self.fetch_alert_configs(tenant, Some(FilterableIds::MonitorIds(monitor_ids)))
            .await
    }
}

#[async_trait]
#[allow(clippy::needless_lifetimes)] // This is needed for the lifetime of the pool
impl<'a> GetByIDs for AlertConfigRepository<'a> {
    async fn get_by_ids(&mut self, ids: &[Uuid], tenant: &str) -> Result<Vec<AlertConfig>, Error> {
        self.fetch_alert_configs(Some(tenant), Some(FilterableIds::AlertConfigIds(ids)))
            .await
    }
}

#[async_trait]
#[allow(clippy::needless_lifetimes)] // This is needed for the lifetime of the pool
impl<'a> Repository<AlertConfig> for AlertConfigRepository<'a> {
    async fn get(
        &mut self,
        alert_config_id: Uuid,
        tenant: &str,
    ) -> Result<Option<AlertConfig>, Error> {
        let mut connection = get_connection(self.pool).await?;
        let result = connection
            .transaction::<Option<(AlertConfigData, Vec<MonitorAlertConfigData>)>, DieselError, _>(
                |conn| {
                    Box::pin(async move {
                        let alert_config_data: Option<AlertConfigData> = build_polymorphic_query!()
                            .filter(
                                alert_config::alert_config_id
                                    .eq(alert_config_id)
                                    .and(alert_config::tenant.eq(tenant)),
                            )
                            .first(conn)
                            .await
                            .optional()?;

                        Ok(if let Some(config_data) = alert_config_data {
                            let monitor_alert_config_datas =
                                MonitorAlertConfigData::belonging_to(&config_data)
                                    .select(MonitorAlertConfigData::as_select())
                                    .load(conn)
                                    .await?;
                            Some((config_data, monitor_alert_config_datas))
                        } else {
                            None
                        })
                    })
                },
            )
            .await
            .map_err(|err| Error::RepositoryError(err.to_string()))?;

        Ok(match result {
            None => None,
            Some((alert_config_data, monitor_alert_configs)) => {
                Some(self.db_to_model(&alert_config_data, &monitor_alert_configs)?)
            }
        })
    }

    async fn all(&mut self, tenant: &str) -> Result<Vec<AlertConfig>, Error> {
        self.fetch_alert_configs(Some(tenant), None).await
    }

    async fn save(&mut self, alert_config: &AlertConfig) -> Result<(), Error> {
        let (alert_config_data, monitor_alert_configs, slack_alert_config_data) =
            NewAlertConfigData::from_model(alert_config);

        // We can do this now as we only support Slack, but when we add more integrations we will
        // need to handle this differently.
        let slack_alert_config_data = slack_alert_config_data.unwrap();

        let mut connection = get_connection(self.pool).await?;
        connection
            .transaction::<(), DieselError, _>(|conn| {
                Box::pin(async {
                    if self.data.contains_key(&alert_config.alert_config_id) {
                        self.update(
                            conn,
                            &alert_config_data,
                            &slack_alert_config_data,
                            &monitor_alert_configs,
                        )
                        .await?;
                    } else {
                        self.insert(
                            conn,
                            &alert_config_data,
                            &slack_alert_config_data,
                            &monitor_alert_configs,
                        )
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
