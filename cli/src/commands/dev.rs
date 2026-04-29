//! `forge dev` — Start the development server and Forge Studio.

use anyhow::Result;
use camino::Utf8PathBuf;
use clap::Args;
use forge_runtime::dev::dev_server::{start_dev_server, DevServerConfig};

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

    let config = DevServerConfig {
        port: args.port,
        studio_port: args.studio_port,
        project_root: Utf8PathBuf::from("."),
    };

    start_dev_server(config).await?;

    Ok(())
}
