// broadcast() — low-level broadcast primitive.
//
// For cases where you need to broadcast to a dynamically-named channel
// topic without a pre-defined Channel instance (e.g., user-specific
// notifications: `broadcast('user:${userId}', payload)`).
//
// Prefer using Channel.define() and channel.broadcast() for type safety.
// This API accepts any JSON-serializable payload.

"use module server"

export async function broadcast(
  topic: string,
  payload: Record<string, unknown>,
  options?: { exclude?: string[] }
): Promise<void> {
  // TODO: Push directly to the WebSocket broker by topic string
}

// broadcastToUser() — helper to send to a user-specific channel.
export async function broadcastToUser(
  userId: string,
  payload: Record<string, unknown>
): Promise<void> {
  return broadcast(`user:${userId}`, payload)
}

// broadcastToRoom() — helper to send to a named room.
export async function broadcastToRoom(
  room: string,
  payload: Record<string, unknown>,
  options?: { exclude?: string[] }
): Promise<void> {
  return broadcast(`room:${room}`, payload, options)
}
