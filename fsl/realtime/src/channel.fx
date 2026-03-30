// Channel.define() — define a typed pub/sub channel.
//
// Channels are named rooms. Multiple clients can subscribe to the same
// channel. The server broadcasts typed messages to all subscribers.
//
// The `authorize` callback is called for each new subscriber. It receives
// the authenticated user (if any) and the channel params (e.g. a room ID).
// Return false or throw to reject the subscription.

"use module server"

import type { ChannelConfig, SubscribeOptions } from './types'

class ChannelInstance<Message extends Record<string, unknown>> {
  readonly name: string
  private config: ChannelConfig<Message>

  constructor(config: ChannelConfig<Message>) {
    this.name = config.name
    this.config = config
  }

  // broadcast() — send a message to all subscribers.
  // Server-side only. Called from server functions or job handlers.
  async broadcast(message: Message, options?: { exclude?: string[] }): Promise<void> {
    // TODO: Push message through the Forge WebSocket broker
  }

  // publish() — alias for broadcast().
  async publish(message: Message): Promise<void> {
    return this.broadcast(message)
  }

  // subscribe() — create a client-side subscription signal.
  // Returns a $signal<Message[]> that appends incoming messages.
  // Client-side only; the compiler ensures this is not called on the server.
  subscribe(options?: SubscribeOptions): { messages: Message[]; connected: boolean } {
    // TODO: Return a reactive subscription backed by the WebSocket connection
    return { messages: [], connected: false }
  }

  // topic() — channel topic with interpolated params (e.g. 'chat:room-42')
  topic(params?: Record<string, string>): string {
    if (!params) return this.name
    return Object.entries(params).reduce(
      (acc, [k, v]) => acc.replace(`:${k}`, v),
      this.name
    )
  }
}

export const Channel = {
  define<Message extends Record<string, unknown>>(
    config: ChannelConfig<Message>
  ): ChannelInstance<Message> {
    return new ChannelInstance<Message>(config)
  },
}
