//! `forge build` — Compile the project for deployment targets.

use anyhow::Result;
use clap::Args;

#[derive(Debug, Args)]
pub struct BuildArgs {
    /// Build a specific named target (default: all targets in forge.toml)
    #[arg(long)]
    pub target: Option<String>,
    /// Enable minification (default: true for release)
    #[arg(long)]
    pub minify: bool,
}

pub async fn run(_args: BuildArgs) -> Result<()> {
    // TODO: Delegate to forge-compiler
    Ok(())
}
