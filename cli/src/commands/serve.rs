//! `forge serve` — Run the production server.

use anyhow::Result;
use clap::Args;
use forge_runtime::server::http_server::{serve, ServerConfig};

#[derive(Debug, Args)]
pub struct ServeArgs {
    #[arg(long, default_value = "3000")]
    pub port: u16,
}

pub async fn run(args: ServeArgs) -> Result<()> {
    crate::output::info(&format!("Starting production server on :{}", args.port));

    let config = ServerConfig {
        port: args.port,
        ..Default::default()
    };

    serve(config).await?;

    Ok(())
}
