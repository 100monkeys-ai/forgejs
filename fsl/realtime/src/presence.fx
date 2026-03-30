// Presence — track which users are currently connected to a channel.
//
// Presence state is maintained by the Forge WebSocket broker. Each
// connected client sends a heartbeat; the broker evicts stale entries.
// Presence changes are themselves broadcast as channel events.

"use module server"

import type { PresenceState, PresenceUser } from './types'

export const Presence = {
  // get() — return the current presence state for a channel.
  async get(channelName: string): Promise<PresenceState> {
    // TODO: Query the WebSocket broker for current subscribers
    return {
      channel: channelName,
      users: [],
      count: 0,
      updatedAt: new Date(),
    }
  },

  // track() — add the current user to a channel's presence.
  // Called automatically when a client subscribes if the channel
  // has presence tracking enabled.
  async track(
    channelName: string,
    user: PresenceUser
  ): Promise<void> {
    // TODO: Register user in the broker's presence map
  },

  // untrack() — remove a user from a channel's presence.
  // Called automatically on disconnect or unsubscribe.
  async untrack(channelName: string, userId: string): Promise<void> {
    // TODO: Remove user from the broker's presence map
  },
}
