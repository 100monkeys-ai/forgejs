//! `forge init` — Initialize Forge in an existing project directory.

use anyhow::Result;
use clap::Args;

#[derive(Debug, Args)]
pub struct InitArgs {
    /// Directory to initialize (default: current directory)
    pub directory: Option<camino::Utf8PathBuf>,
}

pub async fn run(args: InitArgs) -> Result<()> {
    use forge_shared::init::{init_project, InitOptions};

    let dir = args
        .directory
        .unwrap_or_else(|| camino::Utf8PathBuf::from("."));

    let options = InitOptions { target_dir: dir };

    let (initialized_dir, name) = init_project(options)?;

    crate::output::success(&format!(
        "Initialized Forge project '{}' in {}",
        name, initialized_dir
    ));

    Ok(())
}
