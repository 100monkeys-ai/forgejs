//! `forge update` — Update installed Foundry packages.

use anyhow::Result;
use clap::Args;

#[derive(Debug, Args)]
pub struct UpdateArgs {
    /// Update only a specific package
    pub package: Option<String>,
}

pub async fn run(args: UpdateArgs) -> Result<()> {
    foundry_client::resolver::update_packages(args.package).await?;
    Ok(())
}
