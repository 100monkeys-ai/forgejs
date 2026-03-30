// Job.define() — typed job factory.
//
// Each Job is an isolated, serializable unit of work with a named payload
// type. The payload must be JSON-serializable — no functions, no class
// instances, no circular references.

"use module server"

import type { JobConfig, JobRecord, RetryConfig } from './types'

class JobInstance<Payload extends Record<string, unknown>> {
  readonly name: string
  private config: JobConfig<Payload>

  constructor(config: JobConfig<Payload>) {
    this.name = config.name
    this.config = config
  }

  async enqueue(
    payload: Payload,
    options?: { delay?: number; priority?: number }
  ): Promise<JobRecord<Payload>> {
    // TODO: Insert job record into the forge:data jobs table
    const record: JobRecord<Payload> = {
      id: crypto.randomUUID(),
      name: this.name,
      payload,
      status: 'pending',
      attempts: 0,
      maxAttempts: this.config.retry?.attempts ?? 1,
      scheduledAt: options?.delay
        ? new Date(Date.now() + options.delay)
        : new Date(),
      createdAt: new Date(),
    }
    return record
  }

  async cancel(jobId: string): Promise<boolean> {
    // TODO: Mark job as cancelled in the database
    return false
  }

  async status(jobId: string): Promise<JobRecord<Payload> | null> {
    // TODO: Look up job record in the database
    return null
  }

  getHandler(): (payload: Payload) => Promise<void> {
    return this.config.run
  }
}

export const Job = {
  define<Payload extends Record<string, unknown>>(
    config: JobConfig<Payload>
  ): JobInstance<Payload> {
    return new JobInstance<Payload>(config)
  },
}
