// forge:realtime — Forge Standard Library WebSocket pub/sub module
//
// Typed channels for real-time communication between server and clients.
// forge:realtime is only available in the `server` and `edge` deployment
// targets; attempting to use it in a static build is a compile-time error.
//
// Channels carry a typed message payload. The server publishes; clients
// subscribe. Presence tracking reports connected users per channel.
//
// Example — define a channel (app/channels.fx):
//   import { Channel } from 'forge:realtime'
//
//   export const ChatChannel = Channel.define<{
//     message: string
//     userId: string
//     timestamp: number
//   }>({
//     name: 'chat',
//     authorize: async ({ user, params }) => user !== null,
//   })
//
// Example — publish from a server function:
//   import { ChatChannel } from 'app/channels.fx'
//   await ChatChannel.broadcast({ message: 'Hello', userId: user.id, timestamp: Date.now() })
//
// Example — subscribe in a component:
//   import { ChatChannel } from 'app/channels.fx'
//   const $messages = ChatChannel.subscribe()

export { Channel } from './channel.fx'
export { Presence } from './presence.fx'
export { broadcast } from './broadcast.fx'
export type {
  ChannelDefinition,
  ChannelConfig,
  PresenceState,
  PresenceUser,
  SubscribeOptions,
} from './types'
