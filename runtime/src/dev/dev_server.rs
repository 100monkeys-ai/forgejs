//! Development server initialization.
//!
//! Starts the dev server with HMR and Forge Studio.
//! Listens on port 3000 (app) and port 3001 (Studio) by default.

use crate::dev::hot_reload::{hmr_router, HmrMessage, HmrState};
use crate::error::RuntimeError;
use axum::Router;
use camino::Utf8PathBuf;
use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
use tokio::net::TcpListener;
use tokio::sync::mpsc;
use tracing::{error, info};

/// Configuration for the development server.
#[derive(Debug, Clone)]
pub struct DevServerConfig {
    /// Port for the dev server (default: 3000)
    pub port: u16,
    /// Port for Forge Studio (default: 3001)
    pub studio_port: u16,
    /// The project root directory
    pub project_root: Utf8PathBuf,
}

impl Default for DevServerConfig {
    fn default() -> Self {
        Self {
            port: 3000,
            studio_port: 3001,
            project_root: Utf8PathBuf::from("."),
        }
    }
}

/// Start the development server.
pub async fn start_dev_server(config: DevServerConfig) -> Result<(), RuntimeError> {
    let hmr_state = HmrState::new();

    // 1. Setup file watcher
    let (tx, mut rx) = mpsc::channel(100);

    let mut watcher = RecommendedWatcher::new(
        move |res| {
            let _ = tx.blocking_send(res);
        },
        Config::default(),
    )
    .map_err(|e| RuntimeError::Internal(format!("Failed to initialize watcher: {}", e)))?;

    // Watch the src directory
    let src_dir = config.project_root.join("src");
    if src_dir.exists() {
        watcher
            .watch(src_dir.as_std_path(), RecursiveMode::Recursive)
            .map_err(|e| RuntimeError::Internal(format!("Failed to watch src directory: {}", e)))?;
        info!("Watching {} for changes", src_dir);
    } else {
        error!("src directory not found in {}", config.project_root);
    }

    // Spawn a background task to process file watcher events and trigger HMR
    let hmr_state_clone = hmr_state.clone();
    tokio::spawn(async move {
        while let Some(res) = rx.recv().await {
            match res {
                Ok(event) => {
                    // For now, we broadcast a reload on any change.
                    // Later, this will trigger the incremental compiler and only push updates for changed modules.
                    if event.kind.is_modify() || event.kind.is_create() || event.kind.is_remove() {
                        info!("File changed, triggering HMR reload");
                        hmr_state_clone.broadcast(HmrMessage::Reload);
                    }
                }
                Err(e) => error!("Watch error: {:?}", e),
            }
        }
        // Keep watcher alive by moving it into this task
        let _watcher = watcher;
    });

    // 2. Main App Server (incorporates HMR router)
    // TODO: Combine with the actual app router
    let app = hmr_router(hmr_state.clone());
    let addr = format!("0.0.0.0:{}", config.port);
    let listener = TcpListener::bind(&addr).await.map_err(RuntimeError::Io)?;

    // 3. Studio Server
    let studio_app = Router::new().route(
        "/",
        axum::routing::get(|| async { "Forge Studio (Coming soon)" }),
    );
    let studio_addr = format!("0.0.0.0:{}", config.studio_port);
    let studio_listener = TcpListener::bind(&studio_addr)
        .await
        .map_err(RuntimeError::Io)?;

    info!("Dev server listening on {}", addr);
    info!("Forge Studio listening on {}", studio_addr);

    // Run both servers concurrently
    tokio::try_join!(
        async {
            axum::serve(listener, app)
                .await
                .map_err(|e| RuntimeError::Http(e.to_string()))
        },
        async {
            axum::serve(studio_listener, studio_app)
                .await
                .map_err(|e| RuntimeError::Http(e.to_string()))
        }
    )?;

    Ok(())
}
