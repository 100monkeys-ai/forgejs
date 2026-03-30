//! WebSocket handling for forge:realtime channels.
//!
//! The WebSocket server manages connections for the `forge:realtime` module.
//! Each connection subscribes to one or more typed channels. The server
//! broadcasts messages to all subscribers of a channel when a message
//! is published.
//!
//! This handler is only active for `server` and `dev` targets — attempting
//! to use `forge:realtime` in a `static` target is a compile error.
