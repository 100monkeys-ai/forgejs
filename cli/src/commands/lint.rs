//! `forge lint` — Run the Forge linter (Oxlint-based).

use anyhow::Result;
use clap::Args;

#[derive(Debug, Args)]
pub struct LintArgs {
    /// Automatically fix lint errors where possible
    #[arg(long)]
    pub fix: bool,
}

pub async fn run(_args: LintArgs) -> Result<()> {
    // TODO: Run Oxlint with Forge-specific rules
    Ok(())
}
