use diesel::prelude::*;
use diesel::result::Error;
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use uuid::Uuid;

use crate::domain::models::monitor::Monitor;
use crate::infrastructure::db_schema::job;
use crate::infrastructure::db_schema::monitor;
use crate::infrastructure::models::job::JobData;
use crate::infrastructure::models::monitor::MonitorData;

pub struct MonitorRepository<'a> {
    db: &'a mut AsyncPgConnection,
}

impl<'a> MonitorRepository<'a> {
    pub fn new(db: &'a mut AsyncPgConnection) -> Self {
        Self { db }
    }

    pub async fn get(&mut self, monitor_id: Uuid) -> Result<Option<Monitor>, Error> {
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

            Ok(Some((monitor, jobs).into()))
        } else {
            Ok(None)
        }
    }

    pub async fn all(&mut self) -> Result<Vec<Monitor>, Error> {
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
            .map(|(job_datas, monitor_data)| (monitor_data, job_datas).into())
            .collect::<Vec<Monitor>>())
    }

    pub async fn add(&mut self, monitor: &Monitor) -> Result<(), Error> {
        // TODO: Test me
        let (monitor_data, job_datas) = <(MonitorData, Vec<JobData>)>::from(monitor);

        diesel::insert_into(monitor::table)
            .values(&monitor_data)
            .execute(self.db)
            .await?;

        diesel::insert_into(job::table)
            .values(&job_datas)
            .execute(self.db)
            .await?;

        Ok(())
    }

    pub async fn update(&mut self, monitor: &Monitor) -> Result<(), Error> {
        // TODO: Test me
        let (monitor_data, job_datas) = <(MonitorData, Vec<JobData>)>::from(monitor);

        diesel::update(&monitor_data)
            .set(&monitor_data)
            .execute(self.db)
            .await?;

        for j in job_datas {
            diesel::update(&j).set(&j).execute(self.db).await?;
        }

        Ok(())
    }

    pub async fn delete(&mut self, monitor: &Monitor) -> Result<(), Error> {
        // TODO: Test me
        let (monitor_data, _) = <(MonitorData, Vec<JobData>)>::from(monitor);

        diesel::delete(&monitor_data).execute(self.db).await?;

        Ok(())
    }
}
