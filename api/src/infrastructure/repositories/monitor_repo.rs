use std::collections::HashMap;

use async_trait::async_trait;
use diesel::dsl::now;
use diesel::prelude::*;
use diesel::result::Error as DieselError;
use diesel_async::AsyncConnection;
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use uuid::Uuid;

use crate::domain::models::monitor::Monitor;
use crate::errors::Error;
use crate::infrastructure::db_schema::job;
use crate::infrastructure::db_schema::monitor;
use crate::infrastructure::models::job::JobData;
use crate::infrastructure::models::monitor::MonitorData;
use crate::infrastructure::repositories::monitor::GetWithLateJobs;
use crate::infrastructure::repositories::Repository;

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

    fn db_to_monitor(
        &mut self,
        monitor_data: MonitorData,
        job_datas: Vec<JobData>,
    ) -> Result<Monitor, Error> {
        let mon: Monitor = monitor_data.to_model(&job_datas)?;
        self.data.insert(mon.monitor_id, (monitor_data, job_datas));
        Ok(mon)
    }
}

#[async_trait]
impl<'a> GetWithLateJobs for MonitorRepository<'a> {
    async fn get_with_late_jobs(&mut self) -> Result<Vec<Monitor>, Error> {
        let result = self
            .db
            .transaction::<(Vec<MonitorData>, Vec<JobData>), DieselError, _>(|conn| {
                Box::pin(async move {
                    let in_progress_condition =
                        job::end_time.is_null().and(now.gt(job::max_end_time));
                    let finished_condition = job::end_time
                        .is_not_null()
                        .and(job::end_time.assume_not_null().gt(job::max_end_time));

                    // Get all late jobs.
                    let monitor_datas: Vec<MonitorData> = monitor::table
                        .inner_join(job::table)
                        .filter(in_progress_condition.or(finished_condition))
                        .select(MonitorData::as_select())
                        .distinct_on(monitor::monitor_id)
                        .load(conn)
                        .await?;

                    let job_datas = JobData::belonging_to(&monitor_datas)
                        .select(JobData::as_select())
                        .order(job::start_time.desc())
                        .load(conn)
                        .await?;

                    Ok((monitor_datas, job_datas))
                })
            })
            .await;

        match result {
            Err(e) => Err(Error::RepositoryError(e.to_string())),
            Ok((monitor_datas, job_datas)) => Ok(job_datas
                .grouped_by(&monitor_datas)
                .into_iter()
                .zip(monitor_datas)
                .map(|(job_datas, monitor_data)| self.db_to_monitor(monitor_data, job_datas))
                .collect::<Result<Vec<Monitor>, Error>>()?),
        }
    }
}

#[async_trait]
impl<'a> Repository<Monitor> for MonitorRepository<'a> {
    async fn get(&mut self, monitor_id: Uuid) -> Result<Option<Monitor>, Error> {
        let result = self
            .db
            .transaction::<Option<(MonitorData, Vec<JobData>)>, DieselError, _>(|conn| {
                Box::pin(async move {
                    let monitor_data = monitor::table
                        .select(MonitorData::as_select())
                        .find(monitor_id)
                        .first(conn)
                        .await
                        .optional()?;

                    Ok(if let Some(monitor) = monitor_data {
                        let jobs = JobData::belonging_to(&monitor)
                            .select(JobData::as_select())
                            .order(job::start_time.desc())
                            .load(conn)
                            .await?;
                        Some((monitor, jobs))
                    } else {
                        None
                    })
                })
            })
            .await;

        match result {
            Err(e) => Err(Error::RepositoryError(e.to_string())),
            Ok(None) => Ok(None),
            Ok(Some((monitor_data, job_datas))) => {
                Ok(Some(self.db_to_monitor(monitor_data, job_datas)?))
            }
        }
    }

    async fn all(&mut self) -> Result<Vec<Monitor>, Error> {
        let result = self
            .db
            .transaction::<(Vec<MonitorData>, Vec<JobData>), DieselError, _>(|conn| {
                Box::pin(async move {
                    let all_monitor_data = monitor::dsl::monitor
                        .select(MonitorData::as_select())
                        .load(conn)
                        .await?;

                    let jobs = JobData::belonging_to(&all_monitor_data)
                        .select(JobData::as_select())
                        .order(job::start_time.desc())
                        .load(conn)
                        .await?;

                    Ok((all_monitor_data, jobs))
                })
            })
            .await;

        match result {
            Err(e) => Err(Error::RepositoryError(e.to_string())),
            Ok((monitor_datas, job_datas)) => Ok(job_datas
                .grouped_by(&monitor_datas)
                .into_iter()
                .zip(monitor_datas)
                .map(|(job_datas, monitor_data)| self.db_to_monitor(monitor_data, job_datas))
                .collect::<Result<Vec<Monitor>, Error>>()?),
        }
    }

    async fn save(&mut self, monitor: &Monitor) -> Result<(), Error> {
        let cached_data = self.data.get(&monitor.monitor_id);

        // Clone the monitor and job data to avoid borrowing issues.
        // let (monitor_data_clone, job_datas_clone) = (monitor_data.clone(), job_datas.clone());
        let result = self
            .db
            .transaction::<(MonitorData, Vec<JobData>), DieselError, _>(|conn| {
                Box::pin(async move {
                    let (monitor_data, job_datas) = <(MonitorData, Vec<JobData>)>::from(monitor);
                    if let Some(cached) = cached_data {
                        diesel::update(&monitor_data)
                            .set(&monitor_data)
                            .execute(conn)
                            .await?;

                        let job_ids = &cached.1.iter().map(|j| j.job_id).collect::<Vec<Uuid>>();
                        for j in &job_datas {
                            // TODO: Handle jobs being deleted. Don't need to worry about this for
                            // now since there isn't anything that deletes jobs within monitors.
                            if job_ids.contains(&j.job_id) {
                                diesel::update(j).set(j).execute(conn).await?;
                            } else {
                                diesel::insert_into(job::table)
                                    .values(j)
                                    .execute(conn)
                                    .await?;
                            }
                        }
                    } else {
                        diesel::insert_into(monitor::table)
                            .values(&monitor_data)
                            .execute(conn)
                            .await?;

                        diesel::insert_into(job::table)
                            .values(&job_datas)
                            .execute(conn)
                            .await?;
                    }

                    Ok((monitor_data, job_datas))
                })
            })
            .await;

        match result {
            Err(e) => Err(Error::RepositoryError(e.to_string())),
            Ok((monitor_data, job_datas)) => {
                self.data
                    .insert(monitor.monitor_id, (monitor_data, job_datas));
                Ok(())
            }
        }
    }

    async fn delete(&mut self, monitor: &Monitor) -> Result<(), Error> {
        let (monitor_data, _) = <(MonitorData, Vec<JobData>)>::from(monitor);

        let result = diesel::delete(&monitor_data).execute(self.db).await;

        match result {
            Err(e) => Err(Error::RepositoryError(e.to_string())),
            Ok(_) => {
                self.data.remove(&monitor.monitor_id);
                Ok(())
            }
        }
    }
}
