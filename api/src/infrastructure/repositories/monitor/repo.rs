use std::collections::HashMap;

use async_trait::async_trait;
use diesel::dsl::now;
use diesel::prelude::*;
use diesel::result::Error as DieselError;
use diesel_async::AsyncConnection;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::domain::models::Monitor;
use crate::errors::Error;
use crate::infrastructure::database::{get_connection, DbPool};
use crate::infrastructure::db_schema::job;
use crate::infrastructure::db_schema::monitor;
use crate::infrastructure::models::job::JobData;
use crate::infrastructure::models::monitor::MonitorData;
use crate::infrastructure::repositories::monitor::GetWithErroneousJobs;
use crate::infrastructure::repositories::Repository;

pub struct MonitorRepository<'a> {
    pool: &'a DbPool,
    data: HashMap<Uuid, (MonitorData, Vec<JobData>)>,
}

#[allow(clippy::needless_lifetimes)] // This is needed for the lifetime of the pool
impl<'a> MonitorRepository<'a> {
    pub fn new(pool: &'a DbPool) -> Self {
        Self {
            pool,
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
#[allow(clippy::needless_lifetimes)] // This is needed for the lifetime of the pool
impl<'a> GetWithErroneousJobs for MonitorRepository<'a> {
    /// Get Monitors with jobs that are late or have finished with an error.
    ///
    /// Note that this method will not return Monitors that have erroneous jobs that have already
    /// been alerted on.
    async fn get_with_erroneous_jobs(&mut self) -> Result<Vec<Monitor>, Error> {
        let mut connection = get_connection(self.pool).await?;
        let (monitor_datas, job_datas) = connection
            .transaction::<(Vec<MonitorData>, Vec<JobData>), DieselError, _>(|conn| {
                Box::pin(async move {
                    let running_late_condition =
                        job::end_time.is_null().and(now.gt(job::max_end_time));
                    let finished_late_condition = job::end_time
                        .is_not_null()
                        .and(job::end_time.assume_not_null().gt(job::max_end_time));

                    // Get all late and errored jobs.
                    let monitor_datas: Vec<MonitorData> = monitor::table
                        .inner_join(job::table)
                        .filter(
                            (job::late_alert_sent
                                .eq(false)
                                .and(running_late_condition.or(finished_late_condition)))
                            .or(job::error_alert_sent
                                .eq(false)
                                .and(job::end_time.is_not_null())
                                .and(job::succeeded.eq(false))),
                        )
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
            .await
            .map_err(|err| Error::RepositoryError(err.to_string()))?;

        Ok(job_datas
            .grouped_by(&monitor_datas)
            .into_iter()
            .zip(monitor_datas)
            .map(|(job_datas, monitor_data)| self.db_to_monitor(monitor_data, job_datas))
            .collect::<Result<Vec<Monitor>, Error>>()?)
    }
}

#[async_trait]
#[allow(clippy::needless_lifetimes)] // This is needed for the lifetime of the pool
impl<'a> Repository<Monitor> for MonitorRepository<'a> {
    async fn get(&mut self, monitor_id: Uuid, tenant: &str) -> Result<Option<Monitor>, Error> {
        let mut connection = get_connection(self.pool).await?;
        let result = connection
            .transaction::<Option<(MonitorData, Vec<JobData>)>, DieselError, _>(|conn| {
                Box::pin(async move {
                    let monitor_data = monitor::table
                        .select(MonitorData::as_select())
                        .filter(
                            monitor::monitor_id
                                .eq(monitor_id)
                                .and(monitor::tenant.eq(tenant)),
                        )
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
            .await
            .map_err(|err| Error::RepositoryError(err.to_string()))?;

        Ok(match result {
            None => None,
            Some((monitor_data, job_datas)) => Some(self.db_to_monitor(monitor_data, job_datas)?),
        })
    }

    async fn all(&mut self, tenant: &str) -> Result<Vec<Monitor>, Error> {
        let mut connection = get_connection(self.pool).await?;
        let (monitor_datas, job_datas) = connection
            .transaction::<(Vec<MonitorData>, Vec<JobData>), DieselError, _>(|conn| {
                Box::pin(async move {
                    let all_monitor_data = monitor::dsl::monitor
                        .select(MonitorData::as_select())
                        .filter(monitor::tenant.eq(tenant))
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
            .await
            .map_err(|err| Error::RepositoryError(err.to_string()))?;

        Ok(job_datas
            .grouped_by(&monitor_datas)
            .into_iter()
            .zip(monitor_datas)
            .map(|(job_datas, monitor_data)| self.db_to_monitor(monitor_data, job_datas))
            .collect::<Result<Vec<Monitor>, Error>>()?)
    }

    async fn save(&mut self, monitor: &Monitor) -> Result<(), Error> {
        let (monitor_data, job_datas) = <(MonitorData, Vec<JobData>)>::from(monitor);

        let mut connection = get_connection(self.pool).await?;
        connection
            .transaction::<(), DieselError, _>(|conn| {
                Box::pin(async {
                    if let Some(cached) = self.data.get(&monitor.clone().monitor_id) {
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

                    self.data
                        .insert(monitor.monitor_id, (monitor_data, job_datas));
                    Ok(())
                })
            })
            .await
            .map_err(|err| Error::RepositoryError(err.to_string()))
    }

    async fn delete(&mut self, monitor: &Monitor) -> Result<(), Error> {
        let (monitor_data, _) = <(MonitorData, Vec<JobData>)>::from(monitor);

        let mut connection = get_connection(self.pool).await?;
        diesel::delete(&monitor_data)
            .execute(&mut connection)
            .await
            .map_err(|err| Error::RepositoryError(err.to_string()))?;

        self.data.remove(&monitor.monitor_id);
        Ok(())
    }
}
