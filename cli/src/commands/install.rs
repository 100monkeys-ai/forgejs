//! `forge install [package]` — Install packages from the Foundry registry.

use anyhow::Result;
use clap::Args;

#[derive(Debug, Args)]
pub struct InstallArgs {
    /// Package(s) to install (e.g., "jeshua/my-lib")
    pub packages: Vec<String>,
    /// Install as a dev dependency
    #[arg(long, short = 'D')]
    pub dev: bool,
}

pub async fn run(args: InstallArgs) -> Result<()> {
    foundry_client::resolver::install_packages(args.packages, args.dev).await?;
    Ok(())
}
