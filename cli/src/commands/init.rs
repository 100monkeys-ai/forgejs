//! `forge init` — Initialize Forge in an existing project directory.

use clap::Args;
use anyhow::Result;

#[derive(Debug, Args)]
pub struct InitArgs {
    /// Directory to initialize (default: current directory)
    pub directory: Option<camino::Utf8PathBuf>,
}

pub async fn run(_args: InitArgs) -> Result<()> {
    // TODO: Add forge.toml to existing project
    Ok(())
}
