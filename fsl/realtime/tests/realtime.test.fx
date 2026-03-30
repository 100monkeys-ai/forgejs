// forge:realtime tests
import { describe, it, expect } from 'forge:test'
import { Channel } from '../src/channel.fx'
import { broadcast } from '../src/broadcast.fx'

const NotificationsChannel = Channel.define<{
  type: string
  message: string
}>({
  name: 'notifications',
  authorize: async ({ user }) => user !== null,
})

describe('Channel.define', () => {
  it('creates a channel instance with the given name', () => {
    expect(NotificationsChannel.name).toBe('notifications')
  })
})

describe('channel.topic', () => {
  it('returns the channel name when no params given', () => {
    expect(NotificationsChannel.topic()).toBe('notifications')
  })
})

describe('channel.subscribe', () => {
  it('returns an initial empty subscription state', () => {
    const sub = NotificationsChannel.subscribe()
    expect(sub.messages).toHaveLength(0)
    expect(sub.connected).toBe(false)
  })
})

describe('broadcast', () => {
  it('resolves without error for a valid topic and payload', async () => {
    await expect(
      broadcast('test:channel', { type: 'ping', ts: Date.now() })
    ).resolves.toBe(undefined)
  })
})
