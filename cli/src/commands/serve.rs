//! `forge serve` — Run the production server.

use anyhow::Result;
use clap::Args;

#[derive(Debug, Args)]
pub struct ServeArgs {
    #[arg(long, default_value = "3000")]
    pub port: u16,
}

pub async fn run(_args: ServeArgs) -> Result<()> {
    // TODO: Delegate to forge-runtime HTTP server
    Ok(())
}
