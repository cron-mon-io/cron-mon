use std::env;

use testcontainers::{runners::AsyncRunner, ContainerAsync, ImageExt};
use testcontainers_modules::postgres::Postgres;

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
