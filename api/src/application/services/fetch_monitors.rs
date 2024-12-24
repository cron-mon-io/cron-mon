use crate::domain::models::Monitor;
use crate::errors::Error;
use crate::infrastructure::repositories::Repository;

pub struct FetchMonitorsService<'a, T: Repository<Monitor>, F: Fn(&mut [Monitor])> {
    repo: T,
    order_monitors: &'a F,
}

impl<'a, T: Repository<Monitor>, F: Fn(&mut [Monitor])> FetchMonitorsService<'a, T, F> {
    pub fn new(repo: T, order_monitors: &'a F) -> Self {
        Self {
            repo,
            order_monitors,
        }
    }

    pub async fn fetch_all(&mut self, tenant: &str) -> Result<Vec<Monitor>, Error> {
        let mut monitors = self.repo.all(tenant).await?;

        (self.order_monitors)(&mut monitors);

        Ok(monitors)
    }
}

#[cfg(test)]
mod tests {
    use mockall::predicate::*;

    use test_utils::gen_uuid;

    use crate::infrastructure::repositories::MockRepository;

    use super::{FetchMonitorsService, Monitor};

    fn order_monitors(monitors: &mut [Monitor]) {
        monitors.sort_by(|lh_mon: &Monitor, rh_mon: &Monitor| lh_mon.name.cmp(&rh_mon.name));
    }

    #[tokio::test]
    async fn test_fetch_job_service() {
        let mut mock = MockRepository::new();
        mock.expect_all()
            .once()
            .with(eq("tenant"))
            .returning(move |_| {
                Ok(vec![
                    Monitor {
                        monitor_id: gen_uuid("41ebffb4-a188-48e9-8ec1-61380085cde3"),
                        tenant: "tenant".to_owned(),
                        name: "foo".to_owned(),
                        expected_duration: 300,
                        grace_duration: 100,
                        jobs: vec![],
                    },
                    Monitor {
                        monitor_id: gen_uuid("91bf0865-b1b2-447b-93e1-fe047d2bb218"),
                        tenant: "tenant".to_owned(),
                        name: "bar".to_owned(),
                        expected_duration: 300,
                        grace_duration: 100,
                        jobs: vec![],
                    },
                    Monitor {
                        monitor_id: gen_uuid("72ab99e7-d179-4d24-b9a3-cb1a65064a4d"),
                        tenant: "tenant".to_owned(),
                        name: "baz".to_owned(),
                        expected_duration: 300,
                        grace_duration: 100,
                        jobs: vec![],
                    },
                ])
            });
        let mut service = FetchMonitorsService::new(mock, &order_monitors);

        let monitors = service.fetch_all("tenant").await.unwrap();

        let names = monitors
            .iter()
            .map(|monitor| monitor.name.clone())
            .collect::<Vec<String>>();
        assert_eq!(names, vec!["bar", "baz", "foo"]);
    }
}
