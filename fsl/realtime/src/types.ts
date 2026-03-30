import type { User } from '../../auth/src/types'

export interface ChannelConfig<Message extends Record<string, unknown>> {
  name: string
  authorize?: (context: { user: User | null; params?: Record<string, string> }) => boolean | Promise<boolean>
  presence?: boolean
}

export interface ChannelDefinition<Message extends Record<string, unknown>> {
  name: string
  broadcast(message: Message, options?: { exclude?: string[] }): Promise<void>
  subscribe(options?: SubscribeOptions): { messages: Message[]; connected: boolean }
  topic(params?: Record<string, string>): string
}

export interface SubscribeOptions {
  params?: Record<string, string>
  onConnect?: () => void
  onDisconnect?: () => void
  bufferSize?: number
}

export interface PresenceUser {
  id: string
  name?: string
  avatar?: string
  metadata?: Record<string, unknown>
}

export interface PresenceState {
  channel: string
  users: PresenceUser[]
  count: number
  updatedAt: Date
}
