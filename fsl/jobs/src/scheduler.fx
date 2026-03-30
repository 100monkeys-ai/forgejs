// Scheduler — cron-based job scheduling.
//
// Scheduled jobs are registered at startup and enqueue themselves on the
// configured cron expression. The scheduler runs inside the Forge server
// process and is not a separate service.

"use module server"

import type { CronSchedule } from './types'

interface ScheduledJob {
  cron: string
  name: string
  run: () => Promise<void>
  lastRunAt?: Date
  nextRunAt?: Date
}

const scheduledJobs: ScheduledJob[] = []

export const Scheduler = {
  cron(expression: string, name: string, run: () => Promise<void>): void {
    scheduledJobs.push({
      cron: expression,
      name,
      run,
      nextRunAt: computeNextRun(expression),
    })
  },

  getAll(): readonly ScheduledJob[] {
    return scheduledJobs
  },
}

function computeNextRun(expression: string): Date {
  // TODO: Parse cron expression and compute next run time
  // Using a Rust cron parser exposed via FFI
  return new Date(Date.now() + 60_000)
}
