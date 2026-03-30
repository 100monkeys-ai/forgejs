//! Command dispatch for the `forge` CLI.
//!
//! All subcommands are defined here and delegated to their respective modules.

use anyhow::Result;
use clap::{Parser, Subcommand};

pub mod build;
pub mod dev;
pub mod init;
pub mod install;
pub mod lint;
pub mod new;
pub mod publish;
pub mod serve;
pub mod test;
pub mod update;

/// The Forge command-line tool.
///
/// Forge is a Rust-powered, opinionated full-stack JavaScript framework.
/// Use `forge new <name>` to create your first project.
#[derive(Debug, Parser)]
#[command(name = "forge", version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Create a new Forge project
    New(new::NewArgs),
    /// Start the development server and Forge Studio
    Dev(dev::DevArgs),
    /// Compile the project for one or more deployment targets
    Build(build::BuildArgs),
    /// Run the production server
    Serve(serve::ServeArgs),
    /// Run the test suite
    Test(test::TestArgs),
    /// Run the Forge linter
    Lint(lint::LintArgs),
    /// Publish a package to the Foundry registry
    Publish(publish::PublishArgs),
    /// Install packages from the Foundry registry
    Install(install::InstallArgs),
    /// Update installed Foundry packages
    Update(update::UpdateArgs),
    /// Initialize Forge in an existing directory
    Init(init::InitArgs),
}

impl Cli {
    pub async fn run(self) -> Result<()> {
        match self.command {
            Commands::New(args) => new::run(args).await,
            Commands::Dev(args) => dev::run(args).await,
            Commands::Build(args) => build::run(args).await,
            Commands::Serve(args) => serve::run(args).await,
            Commands::Test(args) => test::run(args).await,
            Commands::Lint(args) => lint::run(args).await,
            Commands::Publish(args) => publish::run(args).await,
            Commands::Install(args) => install::run(args).await,
            Commands::Update(args) => update::run(args).await,
            Commands::Init(args) => init::run(args).await,
        }
    }
}
