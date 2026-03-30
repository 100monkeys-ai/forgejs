// forge:storage tests
import { describe, it, expect } from 'forge:test'
import { Storage } from '../src/bucket.fx'

const storage = Storage.configure({
  adapter: { type: 'local', directory: '/tmp/forge-test-storage' },
})

describe('Storage.configure', () => {
  it('returns a storage instance', () => {
    expect(storage).toBeDefined()
    expect(typeof storage.upload).toBe('function')
    expect(typeof storage.download).toBe('function')
    expect(typeof storage.presignedUrl).toBe('function')
  })
})

describe('storage.upload', () => {
  it('assigns a key under the given prefix', async () => {
    const file = new File(['hello'], 'hello.txt', { type: 'text/plain' })
    const ref = await storage.upload('uploads/', file)
    expect(ref.key.startsWith('uploads/')).toBe(true)
    expect(ref.contentType).toBe('text/plain')
  })
})

describe('storage.presignedUrl', () => {
  it('returns a key and future expiry', async () => {
    const result = await storage.presignedUrl('uploads/', { expiresIn: 60 })
    expect(result.key.startsWith('uploads/')).toBe(true)
    expect(result.expiresAt.getTime()).toBeGreaterThan(Date.now())
  })
})
