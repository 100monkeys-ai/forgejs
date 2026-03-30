//! `forge dev` — Start the development server and Forge Studio.

use clap::Args;
use anyhow::Result;

#[derive(Debug, Args)]
pub struct DevArgs {
    /// Port for the dev server (default: 3000)
    #[arg(long, default_value = "3000")]
    pub port: u16,
    /// Port for Forge Studio (default: 3001)
    #[arg(long, default_value = "3001")]
    pub studio_port: u16,
}

pub async fn run(args: DevArgs) -> Result<()> {
    crate::output::info(&format!("Starting dev server on :{}", args.port));
    crate::output::info(&format!("Forge Studio on :{}", args.studio_port));
    // TODO: Delegate to forge-runtime dev server
    Ok(())
}
