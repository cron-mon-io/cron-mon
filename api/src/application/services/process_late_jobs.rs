use crate::infrastructure::repositories::monitor_repo::GetWithLateJobs;

// For some reason we need to implement Sync and Send here to avoid a compilation error where this
// application service is used in the `POST /monitors` route. But we _don't_ need it for the
// `DeleteMonitorSerivce`...?
pub struct ProcessLateJobsService<'a> {
    repo: &'a mut (dyn GetWithLateJobs + Sync + Send),
}

impl<'a> ProcessLateJobsService<'a> {
    pub fn new(repo: &'a mut (dyn GetWithLateJobs + Sync + Send)) -> Self {
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
