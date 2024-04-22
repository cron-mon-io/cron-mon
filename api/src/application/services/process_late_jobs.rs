use crate::infrastructure::repositories::monitor::GetWithLateJobs;

pub struct ProcessLateJobsService<'a, T: GetWithLateJobs> {
    repo: &'a mut T,
}

impl<'a, T: GetWithLateJobs> ProcessLateJobsService<'a, T> {
    pub fn new(repo: &'a mut T) -> Self {
        Self { repo }
    }

    pub async fn process_late_jobs(&mut self) {
        // TODO: Test me
        println!("Beginning check for late Jobs...");
        let monitors_with_late_jobs = self
            .repo
            .get_with_late_jobs()
            .await
            .expect("Failed to retrieve Monitors with late jobs");

        for mon in &monitors_with_late_jobs {
            let late_jobs = mon.late_jobs();
            // TODO: Send notifications here.
            println!(
                "Monitor '{}' ({}) has {} late jobs",
                &mon.name,
                &mon.monitor_id,
                late_jobs.len()
            );
        }

        println!("Check for late Jobs complete\n");
    }
}
