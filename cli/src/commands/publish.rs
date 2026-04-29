//! `forge publish` — Publish a package to the Foundry registry.

use anyhow::Result;
use clap::Args;
use crate::config::CliConfig;

#[derive(Debug, Args)]
pub struct PublishArgs {
    /// Perform a dry run without actually publishing
    #[arg(long)]
    pub dry_run: bool,
}

pub async fn run(args: PublishArgs) -> Result<()> {
    let config = CliConfig::load();
    let registry_url = config.registry_url.unwrap_or_else(|| "https://registry.forgejs.com".to_string());

    let options = foundry_client::publish::PublishOptions {
        dir: camino::Utf8PathBuf::from("."),
        dry_run: args.dry_run,
        registry_url,
        auth_token: config.auth_token,
    };

    foundry_client::publish::publish_package(options).await?;

    Ok(())
}
