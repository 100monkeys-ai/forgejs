//! `forge serve` — Run the production server.

use anyhow::Result;
use clap::Args;

#[derive(Debug, Args)]
pub struct ServeArgs {
    #[arg(long, default_value = "3000")]
    pub port: u16,
}

pub async fn run(args: ServeArgs) -> Result<()> {
    let config = forge_runtime::server::http_server::ServerConfig {
        port: args.port,
        host: "0.0.0.0".to_string(), // Keep default host for now
    };

    forge_runtime::server::http_server::serve(config).await?;

    Ok(())
}
