mod integration_test {
    use cron_mon_api::infrastructure::repositories::All;
    use tokio::test;

    use cron_mon_api::infrastructure::database::establish_connection;
    use cron_mon_api::infrastructure::repositories::monitor_repo::MonitorRepository;

    #[test]
    async fn full_integration_test() {
        // See data seeds for the expected data (/api/src/infrastructure/seeding/seeds.sql)
        let mut conn = establish_connection().await;
        let mut repo = MonitorRepository::new(&mut conn);

        let montiors = repo.all().await.expect("Failed to retrieve monitors");
        assert_eq!(montiors.len(), 5);
    }
}
