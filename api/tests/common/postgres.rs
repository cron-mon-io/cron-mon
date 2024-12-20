use std::env;

use diesel_async::{AsyncPgConnection, RunQueryDsl};
use testcontainers::{runners::AsyncRunner, ContainerAsync, ImageExt};
use testcontainers_modules::postgres::Postgres;

use cron_mon_api::infrastructure::database::{create_connection_pool, DbPool};
use cron_mon_api::infrastructure::db_schema::{
    alert_config, api_key, job, monitor, monitor_alert_config, slack_alert_config,
};
use cron_mon_api::infrastructure::models::{
    alert_config::{MonitorAlertConfigData, NewAlertConfigData, NewSlackAlertConfigData},
    api_key::ApiKeyData,
    job::JobData,
    monitor::MonitorData,
};

pub type PostgresContainer = ContainerAsync<Postgres>;

pub async fn postgres_container() -> PostgresContainer {
    let container = Postgres::default()
        .with_user("cron-mon-api")
        .with_password("itsasecret")
        .with_db_name("cron-mon")
        .with_name("public.ecr.aws/docker/library/postgres")
        .with_tag("16.1")
        .start()
        .await
        .expect("Failed to start Postgres container");

    env::set_var(
        "DATABASE_URL",
        format!(
            "postgres://cron-mon-api:itsasecret@{}:{}/cron-mon",
            container.get_host().await.unwrap(),
            container.get_host_port_ipv4(5432).await.unwrap()
        ),
    );

    container
}

pub async fn seed_db(
    monitor_seeds: &Vec<MonitorData>,
    job_seeds: &Vec<JobData>,
    api_key_seeds: &Vec<ApiKeyData>,
    alert_config_seeds: &(
        Vec<NewAlertConfigData>,
        Vec<NewSlackAlertConfigData>,
        Vec<MonitorAlertConfigData>,
    ),
) -> DbPool {
    let pool = create_connection_pool().expect("Failed to setup DB connection pool");

    let mut conn = pool
        .get()
        .await
        .expect("Failed to retrieve DB connection from the pool");

    delete_existing_data(&mut conn).await;

    diesel::insert_into(monitor::table)
        .values(monitor_seeds)
        .execute(&mut conn)
        .await
        .expect("Failed to seed monitors");

    diesel::insert_into(job::table)
        .values(job_seeds)
        .execute(&mut conn)
        .await
        .expect("Failed to seed jobs");

    diesel::insert_into(api_key::table)
        .values(api_key_seeds)
        .execute(&mut conn)
        .await
        .expect("Failed to seed api_keys");

    let (alert_config_seeds, slack_alert_config_seeds, monitor_alert_config_seeds) =
        alert_config_seeds;
    diesel::insert_into(alert_config::table)
        .values(alert_config_seeds)
        .execute(&mut conn)
        .await
        .expect("Failed to seed alert_configs");

    diesel::insert_into(slack_alert_config::table)
        .values(slack_alert_config_seeds)
        .execute(&mut conn)
        .await
        .expect("Failed to seed slack_alert_configs");

    diesel::insert_into(monitor_alert_config::table)
        .values(monitor_alert_config_seeds)
        .execute(&mut conn)
        .await
        .expect("Failed to seed monitor_alert_config");

    pool
}

async fn delete_existing_data(conn: &mut AsyncPgConnection) {
    diesel::delete(monitor::table)
        .execute(conn)
        .await
        .expect("Failed to delete existing monitor data");

    diesel::delete(job::table)
        .execute(conn)
        .await
        .expect("Failed to delete existing job data");

    diesel::delete(api_key::table)
        .execute(conn)
        .await
        .expect("Failed to delete existing api_key data");

    diesel::delete(alert_config::table)
        .execute(conn)
        .await
        .expect("Failed to delete existing alert_config data");

    diesel::delete(slack_alert_config::table)
        .execute(conn)
        .await
        .expect("Failed to delete existing slack_alert_config data");

    diesel::delete(monitor_alert_config::table)
        .execute(conn)
        .await
        .expect("Failed to delete existing monitor_alert_config data");
}
