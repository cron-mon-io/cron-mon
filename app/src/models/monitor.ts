import type { Job } from './job'

// TODO: Can/ should we make these classes? For Job this would mean that toTimeDelta could be a method,
// making it a view model of some variety.
export type BasicMonitorInformation = {
  name: string
  expected_duration: number
  grace_duration: number
}

export type MonitorInformation = BasicMonitorInformation & {
  monitor_id: string
  last_finished_job: Job | null
  last_started_job: Job | null
}

export type Monitor = MonitorInformation & {
  jobs: Array<Job>
}
