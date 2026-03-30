//! HTTP server: handles incoming requests and routes them to SSR or server functions.
//!
//! The HTTP server is built on [axum](https://crates.io/crates/axum), a
//! type-safe async web framework built on hyper and tower. It serves:
//!
//! - **SSR routes**: renders the Forge app server-side for the requested URL
//! - **RPC endpoints**: handles calls to server functions (at `/_forge/rpc/*`)
//! - **Static assets**: serves the compiled JS/CSS bundles and static files
//! - **WebSocket**: handles realtime channel connections (at `/_forge/ws`)

pub mod http_server;
pub mod router;
pub mod ssr;
pub mod websocket;
