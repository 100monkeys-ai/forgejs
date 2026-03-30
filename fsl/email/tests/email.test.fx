// forge:email tests
import { describe, it, expect } from 'forge:test'
import { configureEmail, sendEmail } from '../src/send.fx'

describe('configureEmail', () => {
  it('configures the email adapter', () => {
    expect(() => {
      configureEmail({
        defaultFrom: 'noreply@example.com',
        send: async (msg) => ({ messageId: 'test-id' }),
      })
    }).not.toThrow()
  })
})

describe('sendEmail', () => {
  it('throws when no adapter is configured', async () => {
    // Reset module state for this test (adapter starts null in a fresh context)
    const { sendEmail: freshSend } = await import('../src/send.fx')
    await expect(freshSend({
      to: 'user@example.com',
      subject: 'Test',
      template: () => null,
      props: {},
    })).rejects.toThrow('not configured')
  })
})
