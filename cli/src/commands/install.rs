//! `forge install [package]` — Install packages from the Foundry registry.

use clap::Args;
use anyhow::Result;

#[derive(Debug, Args)]
pub struct InstallArgs {
    /// Package(s) to install (e.g., "jeshua/my-lib")
    pub packages: Vec<String>,
    /// Install as a dev dependency
    #[arg(long, short = 'D')]
    pub dev: bool,
}

pub async fn run(_args: InstallArgs) -> Result<()> {
    // TODO: Delegate to foundry-client resolver
    Ok(())
}
