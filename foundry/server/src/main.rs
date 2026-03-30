//! # foundry
//!
//! The Foundry package registry server.
//!
//! The Foundry is Forge's package registry — the server that developers
//! publish packages to and that `forge install` downloads from.
//!
//! The public Foundry registry runs at https://registry.forgejs.com.
//! Organizations can also run a private Foundry server for internal packages.
//!
//! ## Starting the Server
//!
//! ```bash
//! foundry serve --port 8080 --database postgres://...
//! ```

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    // TODO: Parse args, initialize DB, start axum server
    tracing::info!("Foundry registry server starting");
    Ok(())
}
