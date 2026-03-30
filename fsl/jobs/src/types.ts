export type JobStatus = 'pending' | 'running' | 'completed' | 'failed' | 'cancelled'

export interface RetryConfig {
  attempts: number
  backoff: 'linear' | 'exponential' | 'fixed'
  initialDelay?: number
  maxDelay?: number
}

export interface JobConfig<Payload extends Record<string, unknown>> {
  name: string
  run: (payload: Payload) => Promise<void>
  retry?: RetryConfig
  timeout?: number
  queue?: string
}

export interface JobRecord<Payload extends Record<string, unknown>> {
  id: string
  name: string
  payload: Payload
  status: JobStatus
  attempts: number
  maxAttempts: number
  scheduledAt: Date
  startedAt?: Date
  completedAt?: Date
  failedAt?: Date
  lastError?: string
  createdAt: Date
}

export interface JobDefinition<Payload extends Record<string, unknown>> {
  name: string
  enqueue(payload: Payload, options?: { delay?: number; priority?: number }): Promise<JobRecord<Payload>>
  cancel(jobId: string): Promise<boolean>
  status(jobId: string): Promise<JobRecord<Payload> | null>
}

export interface CronSchedule {
  expression: string
  timezone?: string
}
