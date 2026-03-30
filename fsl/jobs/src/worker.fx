// startWorker() — the job consumer process.
//
// The worker polls the job table, claims pending jobs, executes their
// handlers, and updates their status. It runs in the same process as the
// Forge server but on a dedicated async task.
//
// In production, forge automatically starts the worker unless
// FORGE_DISABLE_WORKER=true is set (for deployments that run a dedicated
// worker process separately).

"use module server"

import type { JobRecord } from './types'

export interface WorkerConfig {
  concurrency?: number
  pollInterval?: number
  queues?: string[]
}

export async function startWorker(config: WorkerConfig = {}): Promise<{ stop: () => Promise<void> }> {
  const concurrency = config.concurrency ?? 5
  const pollInterval = config.pollInterval ?? 1000
  let running = true

  const loop = async () => {
    while (running) {
      await processBatch(concurrency)
      await delay(pollInterval)
    }
  }

  loop()

  return {
    async stop() {
      running = false
    },
  }
}

async function processBatch(concurrency: number): Promise<void> {
  // TODO: Claim up to `concurrency` pending jobs from the database
  // Execute each job handler concurrently
  // Update status to 'completed' or 'failed' after execution
  // Schedule retry if failed and attempts < maxAttempts
}

async function delay(ms: number): Promise<void> {
  return new Promise(resolve => setTimeout(resolve, ms))
}
