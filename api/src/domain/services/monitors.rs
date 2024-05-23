use chrono::offset::Utc;
use chrono::TimeZone;

use crate::domain::models::monitor::Monitor;

/// Orders the monitors by the time of the last job that was started. Monitors that don't have any
/// jobs yet should be at the end of the list.
pub fn order_monitors_by_last_started_job(monitors: &mut Vec<Monitor>) {
    monitors.sort_by(|lh_mon: &Monitor, rh_mon: &Monitor| {
        let earliest_time = Utc
            .with_ymd_and_hms(1970, 1, 1, 0, 1, 1)
            .unwrap()
            .naive_utc();

        let lh_t = if let Some(job) = lh_mon.last_started_job() {
            &job.start_time
        } else {
            &earliest_time
        };

        let rh_t = if let Some(job) = rh_mon.last_started_job() {
            &job.start_time
        } else {
            &earliest_time
        };

        rh_t.cmp(&lh_t)
    });
}

#[cfg(test)]
mod tests {
    use std::vec;

    use rstest::*;

    use crate::domain::models::job::Job;
    use test_utils::{gen_datetime, gen_uuid};

    use super::{order_monitors_by_last_started_job, Monitor};

    #[fixture]
    fn monitors() -> Vec<Monitor> {
        vec![
            Monitor {
                monitor_id: gen_uuid("c1bf0515-df39-448b-aa95-686360a33b36"),
                name: "db-backup.py".to_owned(),
                expected_duration: 1800,
                grace_duration: 600,
                jobs: vec![],
            },
            Monitor {
                monitor_id: gen_uuid("cc6cf74e-b25d-4c8c-94a6-914e3f139c14"),
                name: "generate-orders.sh".to_owned(),
                expected_duration: 3600,
                grace_duration: 1200,
                jobs: vec![
                    Job::new(
                        gen_uuid("8106bab7-d643-4ede-bd92-60c79f787344"),
                        gen_datetime("2024-05-01T00:30:00"),
                        gen_datetime("2024-05-01T01:10:00"),
                        Some(gen_datetime("2024-05-01T00:49:00")),
                        Some(true),
                        Some("Orders generated successfully".to_owned()),
                    ),
                    Job::new(
                        gen_uuid("c1893113-66d7-4707-9a51-c8be46287b2c"),
                        gen_datetime("2024-05-01T00:00:00"),
                        gen_datetime("2024-05-01T00:40:00"),
                        Some(gen_datetime("2024-05-01T00:39:00")),
                        Some(false),
                        Some("Failed to generate orders".to_owned()),
                    ),
                ],
            },
            Monitor {
                monitor_id: gen_uuid("d1f3b3b4-0b3b-4b3b-8b3b-3b3b3b3b3b3b"),
                name: "send-emails.sh".to_owned(),
                expected_duration: 7200,
                grace_duration: 1800,
                jobs: vec![Job::new(
                    gen_uuid("9d4e2d69-af63-4c1e-8639-60cb2683aee5"),
                    gen_datetime("2024-05-01T00:20:00"),
                    gen_datetime("2024-05-01T01:00:00"),
                    None,
                    None,
                    None,
                )],
            },
        ]
    }

    #[rstest]
    fn test_order_monitors_by_last_started_job(mut monitors: Vec<Monitor>) {
        order_monitors_by_last_started_job(&mut monitors);

        let names = monitors
            .iter()
            .map(|monitor| monitor.name.clone())
            .collect::<Vec<String>>();
        assert_eq!(
            names,
            vec!["generate-orders.sh", "send-emails.sh", "db-backup.py"]
        );
    }
}
