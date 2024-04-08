use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::result::Error;
use uuid::Uuid;

use crate::domain::models::monitor::Monitor;
use crate::infrastructure::db_schema::job;
use crate::infrastructure::db_schema::monitor;
use crate::infrastructure::models::job::JobData;
use crate::infrastructure::models::monitor::MonitorData;

pub struct MonitorRepository<'a> {
    db: &'a mut PgConnection,
}

impl<'a> MonitorRepository<'a> {
    pub fn new(db: &'a mut PgConnection) -> Self {
        Self { db }
    }

    pub fn get(&mut self, monitor_id: Uuid) -> Result<Option<Monitor>, Error> {
        // TODO: Test me
        let monitor_data = monitor::table
            .select(MonitorData::as_select())
            .find(monitor_id)
            .first(self.db)
            .optional()?;

        if let Some(monitor) = monitor_data {
            let jobs = JobData::belonging_to(&monitor)
                .select(JobData::as_select())
                .load(self.db)?;
            // TODO handle monitors without jobs.

            Ok(Some((monitor, jobs).into()))
        } else {
            Ok(None)
        }
    }

    pub fn all(&mut self) -> Result<Vec<Monitor>, Error> {
        // TODO: Test me
        let all_monitor_data = monitor::dsl::monitor
            .select(MonitorData::as_select())
            .load(self.db)?;

        let jobs = JobData::belonging_to(&all_monitor_data)
            .select(JobData::as_select())
            .load(self.db)?;

        Ok(jobs
            .grouped_by(&all_monitor_data)
            .into_iter()
            .zip(all_monitor_data)
            .map(|(job_datas, monitor_data)| (monitor_data, job_datas).into())
            .collect::<Vec<Monitor>>())
    }

    pub fn add(&mut self, monitor: &Monitor) -> Result<(), Error> {
        // TODO: Test me
        let (monitor_data, job_datas) = <(MonitorData, Vec<JobData>)>::from(monitor);

        diesel::insert_into(monitor::table)
            .values(&monitor_data)
            .execute(self.db)?;

        diesel::insert_into(job::table)
            .values(&job_datas)
            .execute(self.db)?;

        Ok(())
    }

    pub fn delete(&mut self, monitor: &Monitor) -> Result<(), Error> {
        // TODO: Test me
        let (monitor_data, _) = <(MonitorData, Vec<JobData>)>::from(monitor);

        diesel::delete(&monitor_data).execute(self.db)?;

        Ok(())
    }
}
