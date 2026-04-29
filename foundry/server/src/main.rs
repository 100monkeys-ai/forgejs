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

use clap::{Parser, Subcommand};
use foundry_server::api::router;
use sqlx::postgres::PgPoolOptions;
use std::net::SocketAddr;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the Foundry server
    Serve {
        /// Port to listen on
        #[arg(long, default_value_t = 8080)]
        port: u16,

        /// Database connection string
        #[arg(long, env = "DATABASE_URL")]
        database: String,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Serve { port, database } => {
            tracing::info!("Foundry registry server starting on port {}", port);

            // Initialize DB
            tracing::info!("Connecting to database...");
            let pool = PgPoolOptions::new()
                .max_connections(5)
                .connect(&database)
                .await?;

            // Setup app router
            let app = router(pool);

            // Start axum server
            let addr = SocketAddr::from(([0, 0, 0, 0], port));
            let listener = tokio::net::TcpListener::bind(addr).await?;

            tracing::info!("Listening on {}", addr);
            axum::serve(listener, app).await?;
        }
    }

    Ok(())
}
