// forge:jobs tests
import { describe, it, expect } from 'forge:test'
import { Job } from '../src/queue.fx'

const TestJob = Job.define<{ message: string }>({
  name: 'test-job',
  retry: { attempts: 3, backoff: 'exponential' },
  async run({ message }) {
    // no-op in tests
  },
})

describe('Job.define', () => {
  it('creates a job instance with the given name', () => {
    expect(TestJob.name).toBe('test-job')
  })
})

describe('TestJob.enqueue', () => {
  it('returns a job record with pending status', async () => {
    const record = await TestJob.enqueue({ message: 'hello' })
    expect(record.name).toBe('test-job')
    expect(record.status).toBe('pending')
    expect(record.payload.message).toBe('hello')
    expect(record.maxAttempts).toBe(3)
  })

  it('assigns a unique id to each job', async () => {
    const a = await TestJob.enqueue({ message: 'a' })
    const b = await TestJob.enqueue({ message: 'b' })
    expect(a.id).not.toBe(b.id)
  })
})
