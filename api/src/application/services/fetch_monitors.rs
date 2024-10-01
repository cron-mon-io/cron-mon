use crate::domain::models::monitor::Monitor;
use crate::errors::Error;
use crate::infrastructure::repositories::All;

pub struct FetchMonitorsService<'a, T: All<Monitor>, F: Fn(&mut [Monitor])> {
    repo: T,
    order_monitors: &'a F,
}

impl<'a, T: All<Monitor>, F: Fn(&mut [Monitor])> FetchMonitorsService<'a, T, F> {
    pub fn new(repo: T, order_monitors: &'a F) -> Self {
        Self {
            repo,
            order_monitors,
        }
    }

    pub async fn fetch_all(&mut self) -> Result<Vec<Monitor>, Error> {
        let mut monitors = self.repo.all().await?;

        (self.order_monitors)(&mut monitors);

        Ok(monitors)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use rstest::{fixture, rstest};
    use uuid::Uuid;

    use test_utils::gen_uuid;

    use crate::infrastructure::repositories::test_repo::{to_hashmap, TestRepository};

    use super::{FetchMonitorsService, Monitor};

    fn order_monitors(monitors: &mut [Monitor]) {
        monitors.sort_by(|lh_mon: &Monitor, rh_mon: &Monitor| lh_mon.name.cmp(&rh_mon.name));
    }

    #[fixture]
    fn data() -> HashMap<Uuid, Monitor> {
        to_hashmap(vec![
            Monitor {
                monitor_id: gen_uuid("41ebffb4-a188-48e9-8ec1-61380085cde3"),
                name: "foo".to_owned(),
                expected_duration: 300,
                grace_duration: 100,
                jobs: vec![],
            },
            Monitor {
                monitor_id: gen_uuid("91bf0865-b1b2-447b-93e1-fe047d2bb218"),
                name: "bar".to_owned(),
                expected_duration: 300,
                grace_duration: 100,
                jobs: vec![],
            },
            Monitor {
                monitor_id: gen_uuid("72ab99e7-d179-4d24-b9a3-cb1a65064a4d"),
                name: "baz".to_owned(),
                expected_duration: 300,
                grace_duration: 100,
                jobs: vec![],
            },
        ])
    }

    #[rstest]
    #[tokio::test]
    async fn test_fetch_job_service(mut data: HashMap<Uuid, Monitor>) {
        let mut service =
            FetchMonitorsService::new(TestRepository::new(&mut data), &order_monitors);

        let monitors = service.fetch_all().await.unwrap();

        let names = monitors
            .iter()
            .map(|monitor| monitor.name.clone())
            .collect::<Vec<String>>();
        assert_eq!(names, vec!["bar", "baz", "foo"]);
    }
}
