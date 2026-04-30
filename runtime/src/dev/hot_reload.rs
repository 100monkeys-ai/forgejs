//! Hot module replacement (HMR) implementation.
//!
//! When a source file changes, the HMR system:
//! 1. Recompiles the changed module (incrementally)
//! 2. Pushes the new module source to connected browsers via a SSE stream
//! 3. The browser's HMR runtime replaces the module in-place if possible,
//!    or triggers a full reload if the module graph changed structurally

use axum::{
    extract::State,
    response::sse::{Event, Sse},
    routing::get,
    Router,
};
use futures::stream::Stream;
use std::convert::Infallible;
use tokio::sync::broadcast;
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::StreamExt;

/// Message sent over the HMR broadcast channel.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(tag = "type", content = "payload")]
pub enum HmrMessage {
    /// A module was updated and should be re-evaluated.
    Update {
        /// The path of the module that was updated.
        path: String,
        /// The new module source code.
        code: String,
    },
    /// The application needs a full reload (e.g. structural change).
    Reload,
}

/// State shared across HMR SSE connections.
#[derive(Clone)]
pub struct HmrState {
    /// Broadcast channel for pushing updates to connected clients.
    pub tx: broadcast::Sender<HmrMessage>,
}

impl Default for HmrState {
    fn default() -> Self {
        Self::new()
    }
}

impl HmrState {
    /// Create a new HmrState.
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(100);
        Self { tx }
    }

    /// Broadcast an update to all connected clients.
    pub fn broadcast(&self, message: HmrMessage) {
        // Ignore send errors (happens when no clients are connected)
        let _ = self.tx.send(message);
    }
}

/// Create the axum router for the HMR endpoint.
pub fn hmr_router(state: HmrState) -> Router<()> {
    Router::new()
        .route("/_forge/hmr", get(hmr_endpoint))
        .with_state(state)
}

/// SSE endpoint handler for HMR connections.
async fn hmr_endpoint(
    State(state): State<HmrState>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let rx = state.tx.subscribe();
    let stream = BroadcastStream::new(rx).filter_map(|msg| {
        match msg {
            Ok(msg) => match serde_json::to_string(&msg) {
                Ok(json) => Some(Ok(Event::default().data(json))),
                Err(e) => {
                    tracing::error!("Failed to serialize HMR message: {}", e);
                    None
                }
            },
            // Ignore lag errors
            Err(_) => None,
        }
    });

    Sse::new(stream).keep_alive(axum::response::sse::KeepAlive::new())
}
