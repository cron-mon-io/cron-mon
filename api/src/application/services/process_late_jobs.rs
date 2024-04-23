use crate::infrastructure::notify::NotifyLateJob;
use crate::infrastructure::repositories::monitor::GetWithLateJobs;

pub struct ProcessLateJobsService<'a, Repo: GetWithLateJobs, Notifier: NotifyLateJob> {
    repo: &'a mut Repo,
    notifier: &'a mut Notifier,
}

impl<'a, Repo: GetWithLateJobs, Notifier: NotifyLateJob>
    ProcessLateJobsService<'a, Repo, Notifier>
{
    pub fn new(repo: &'a mut Repo, notifier: &'a mut Notifier) -> Self {
        Self { repo, notifier }
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
            for late_job in mon.late_jobs() {
                self.notifier
                    .notify_late_job(&mon.name, late_job)
                    .expect("Failed to notify job is late");
            }
        }

        println!("Check for late Jobs complete\n");
    }
}
