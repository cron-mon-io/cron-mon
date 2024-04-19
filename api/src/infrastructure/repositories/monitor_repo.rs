use std::collections::HashMap;

use async_trait::async_trait;
use diesel::dsl::now;
use diesel::prelude::*;
use diesel::result::Error;
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use uuid::Uuid;

use crate::domain::models::monitor::Monitor;
use crate::infrastructure::db_schema::job;
use crate::infrastructure::db_schema::monitor;
use crate::infrastructure::models::job::JobData;
use crate::infrastructure::models::monitor::MonitorData;

use crate::infrastructure::repositories::{All, Delete, Get, Save};

pub struct MonitorRepository<'a> {
    db: &'a mut AsyncPgConnection,
    data: HashMap<Uuid, (MonitorData, Vec<JobData>)>,
}

impl<'a> MonitorRepository<'a> {
    pub fn new(db: &'a mut AsyncPgConnection) -> Self {
        Self {
            db,
            data: HashMap::new(),
        }
    }

    fn db_to_monitor(&mut self, monitor_data: MonitorData, job_datas: Vec<JobData>) -> Monitor {
        let mon: Monitor = (&monitor_data, &job_datas).into();
        self.data.insert(mon.monitor_id, (monitor_data, job_datas));
        mon
    }
}

#[async_trait]
pub trait GetWithLateJobs {
    async fn get_with_late_jobs(&mut self) -> Result<Vec<Monitor>, Error>;
}

#[async_trait]
impl<'a> GetWithLateJobs for MonitorRepository<'a> {
    async fn get_with_late_jobs(&mut self) -> Result<Vec<Monitor>, Error> {
        let in_progress_condition = job::end_time.is_null().and(now.gt(job::max_end_time));
        let finished_condition = job::end_time
            .is_not_null()
            .and(job::end_time.assume_not_null().gt(job::max_end_time));
        // Get all late jobs.
        let late_jobs: Vec<JobData> = job::dsl::job
            .inner_join(monitor::table)
            .filter(in_progress_condition.or(finished_condition))
            .select(JobData::as_select())
            .load(self.db)
            .await?;

        // Get the monitors that the late jobs belong too.
        // TODO: Refactor the below as it's very close to what we're doing in `all`.
        let monitor_datas = monitor::table
            .select(MonitorData::as_select())
            .filter(
                monitor::monitor_id.eq_any(
                    late_jobs
                        .iter()
                        .map(|j| j.monitor_id)
                        .collect::<Vec<Uuid>>(),
                ),
            )
            .load(self.db)
            .await?;

        let jobs = JobData::belonging_to(&monitor_datas)
            .select(JobData::as_select())
            .load(self.db)
            .await?;

        Ok(jobs
            .grouped_by(&monitor_datas)
            .into_iter()
            .zip(monitor_datas)
            .map(|(job_datas, monitor_data)| self.db_to_monitor(monitor_data, job_datas))
            .collect::<Vec<Monitor>>())
    }
}

#[async_trait]
impl<'a> Get<Monitor> for MonitorRepository<'a> {
    async fn get(&mut self, monitor_id: Uuid) -> Result<Option<Monitor>, Error> {
        // TODO: Test me
        let monitor_data = monitor::table
            .select(MonitorData::as_select())
            .find(monitor_id)
            .first(self.db)
            .await
            .optional()?;

        if let Some(monitor) = monitor_data {
            let jobs = JobData::belonging_to(&monitor)
                .select(JobData::as_select())
                .load(self.db)
                .await?;
            // TODO handle monitors without jobs.
            Ok(Some(self.db_to_monitor(monitor, jobs)))
        } else {
            Ok(None)
        }
    }
}

#[async_trait]
impl<'a> All<Monitor> for MonitorRepository<'a> {
    async fn all(&mut self) -> Result<Vec<Monitor>, Error> {
        // TODO: Test me
        let all_monitor_data = monitor::dsl::monitor
            .select(MonitorData::as_select())
            .load(self.db)
            .await?;

        let jobs = JobData::belonging_to(&all_monitor_data)
            .select(JobData::as_select())
            .load(self.db)
            .await?;

        Ok(jobs
            .grouped_by(&all_monitor_data)
            .into_iter()
            .zip(all_monitor_data)
            .map(|(job_datas, monitor_data)| self.db_to_monitor(monitor_data, job_datas))
            .collect::<Vec<Monitor>>())
    }
}

#[async_trait]
impl<'a> Save<Monitor> for MonitorRepository<'a> {
    async fn save(&mut self, monitor: &Monitor) -> Result<(), Error> {
        // TODO: Test me
        let (monitor_data, job_datas) = <(MonitorData, Vec<JobData>)>::from(monitor);
        let cached_data = self.data.get(&monitor.monitor_id);
        if let Some(cached) = cached_data {
            diesel::update(&monitor_data)
                .set(&monitor_data)
                .execute(self.db)
                .await?;

            let job_ids = &cached.1.iter().map(|j| j.job_id).collect::<Vec<Uuid>>();
            for j in &job_datas {
                if job_ids.contains(&j.job_id) {
                    diesel::update(j).set(j).execute(self.db).await?;
                } else {
                    diesel::insert_into(job::table)
                        .values(j)
                        .execute(self.db)
                        .await?;
                }
            }
        } else {
            diesel::insert_into(monitor::table)
                .values(&monitor_data)
                .execute(self.db)
                .await?;

            diesel::insert_into(job::table)
                .values(&job_datas)
                .execute(self.db)
                .await?;
        }

        self.data
            .insert(monitor.monitor_id, (monitor_data, job_datas));

        Ok(())
    }
}

#[async_trait]
impl<'a> Delete<Monitor> for MonitorRepository<'a> {
    async fn delete(&mut self, monitor: &Monitor) -> Result<(), Error> {
        // TODO: Test me
        let (monitor_data, _) = <(MonitorData, Vec<JobData>)>::from(monitor);

        diesel::delete(&monitor_data).execute(self.db).await?;

        Ok(())
    }
}
