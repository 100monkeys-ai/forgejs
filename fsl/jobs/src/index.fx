// forge:jobs — Forge Standard Library background jobs module
//
// Typed background job queue with persistent storage via forge:data.
// Jobs are defined with a payload type and a handler function. The queue
// is durable — jobs survive server restarts. Retries use exponential
// backoff with configurable limits.
//
// Example — app/jobs/SendWelcomeEmail.fx:
//   import { Job } from 'forge:jobs'
//   import { sendEmail } from 'forge:email'
//   import WelcomeEmail from 'app/emails/WelcomeEmail.fx'
//
//   export const SendWelcomeEmail = Job.define<{ userId: string; email: string }>({
//     name: 'send-welcome-email',
//     retry: { attempts: 3, backoff: 'exponential' },
//     async run({ userId, email }) {
//       await sendEmail({ to: email, subject: 'Welcome!', template: WelcomeEmail, props: { userId } })
//     }
//   })
//
// Example — enqueue from a server function:
//   import { SendWelcomeEmail } from 'app/jobs/SendWelcomeEmail.fx'
//   await SendWelcomeEmail.enqueue({ userId: user.id, email: user.email })

export { Job } from './queue.fx'
export { Scheduler } from './scheduler.fx'
export { startWorker } from './worker.fx'
export type {
  JobDefinition,
  JobConfig,
  JobRecord,
  JobStatus,
  RetryConfig,
  CronSchedule,
} from './types'
