//! `forge publish` — Publish a package to the Foundry registry.

use anyhow::Result;
use clap::Args;

#[derive(Debug, Args)]
pub struct PublishArgs {
    /// Perform a dry run without actually publishing
    #[arg(long)]
    pub dry_run: bool,
}

pub async fn run(_args: PublishArgs) -> Result<()> {
    // TODO: Delegate to foundry-client publish
    Ok(())
}
