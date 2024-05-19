export type Job = {
  job_id: string
  start_time: Date
  end_time: Date | null
  in_progress: boolean
  late: boolean
  duration: number | null
  succeeded: boolean | null
  output: string | null
}
