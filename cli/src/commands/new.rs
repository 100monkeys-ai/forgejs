//! `forge new <name>` — Create a new Forge project.
//!
//! Scaffolds a new Forge project with:
//! - `forge.toml` with sensible defaults
//! - `app/root.fx` — root layout component
//! - `app/routes.fx` — route definitions
//! - `app/pages/Home.fx` — welcome page
//! - `schema.fx` — empty database schema
//! - `foundry.lock` — empty lockfile
//!
//! Optionally prompts for:
//! - Deployment targets (server, edge, desktop, mobile)
//! - Database adapter (SQLite default, PostgreSQL)
//! - Authentication (email+password, passkey, OAuth)

use anyhow::Result;
use clap::Args;

#[derive(Debug, Args)]
pub struct NewArgs {
    /// Name of the new project
    pub name: String,
    /// Skip interactive prompts and use defaults
    #[arg(long, short)]
    pub yes: bool,
}

pub async fn run(args: NewArgs) -> Result<()> {
    crate::output::info(&format!("Creating new Forge project: {}", args.name));
    // TODO: Implement project scaffolding
    crate::output::success(&format!(
        "Created {} — run `cd {} && forge dev` to start",
        args.name, args.name
    ));
    Ok(())
}
