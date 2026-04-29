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
  // Parse cron expression and compute next run time
  // This uses a built-in TS cron parser fallback as the Rust FFI boundary
  // for deno_core ops is not yet fully initialized/exposed in the runtime context.

  try {
    const parts = expression.trim().split(/\s+/);
    if (parts.length !== 5 && parts.length !== 6) {
      throw new Error('Invalid cron expression length');
    }

    const now = new Date();
    // Start checking from the next minute
    let next = new Date(now.getFullYear(), now.getMonth(), now.getDate(), now.getHours(), now.getMinutes() + 1, 0, 0);

    const matchField = (val: number, field: string): boolean => {
      if (field === '*') return true;
      if (field.includes(',')) {
        return field.split(',').some(part => matchField(val, part));
      }
      if (field.includes('/')) {
        const [range, step] = field.split('/');
        const stepNum = parseInt(step, 10);
        if (range === '*') return val % stepNum === 0;
        if (range.includes('-')) {
            const [start, end] = range.split('-');
            return val >= parseInt(start, 10) && val <= parseInt(end, 10) && (val - parseInt(start, 10)) % stepNum === 0;
        }
        return val % stepNum === 0;
      }
      if (field.includes('-')) {
        const [start, end] = field.split('-');
        return val >= parseInt(start, 10) && val <= parseInt(end, 10);
      }
      return val === parseInt(field, 10);
    };

    const hasSec = parts.length === 6;
    const offset = hasSec ? 1 : 0;
    const minField = parts[0 + offset];
    const hourField = parts[1 + offset];
    const domField = parts[2 + offset];
    const monField = parts[3 + offset];
    const dowField = parts[4 + offset];

    // Search forward up to 5 years
    for (let i = 0; i < 5 * 365 * 24 * 60; i++) {
      const monMatch = matchField(next.getMonth() + 1, monField);
      if (!monMatch) {
          next.setDate(1);
          next.setMonth(next.getMonth() + 1);
          next.setHours(0);
          next.setMinutes(0);
          continue;
      }

      const domMatch = matchField(next.getDate(), domField);
      const dowMatch = matchField(next.getDay(), dowField);

      const dayMatch = (domField === '*' || dowField === '*')
        ? (domMatch && dowMatch)
        : (domMatch || dowMatch);

      if (!dayMatch) {
          next.setDate(next.getDate() + 1);
          next.setHours(0);
          next.setMinutes(0);
          continue;
      }

      const hourMatch = matchField(next.getHours(), hourField);
      if (!hourMatch) {
          next.setHours(next.getHours() + 1);
          next.setMinutes(0);
          continue;
      }

      const minMatch = matchField(next.getMinutes(), minField);
      if (minMatch) {
          return next;
      }

      next.setMinutes(next.getMinutes() + 1);
    }

    return new Date(Date.now() + 60_000);
  } catch (err) {
    return new Date(Date.now() + 60_000);
  }
}
