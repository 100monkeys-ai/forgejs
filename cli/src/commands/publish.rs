//! `forge publish` — Publish a package to the Foundry registry.

use anyhow::Result;
use clap::Args;

#[derive(Debug, Args)]
pub struct PublishArgs {
    /// Perform a dry run without actually publishing
    #[arg(long)]
    pub dry_run: bool,
}

pub async fn run(args: PublishArgs) -> Result<()> {
    use foundry_client::publish::{publish_package, PublishOptions};

    publish_package(PublishOptions {
        dry_run: args.dry_run,
    })
    .await?;

    Ok(())
}
