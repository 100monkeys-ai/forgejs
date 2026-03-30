//! # forge
//!
//! The Forge command-line tool. Entry point for all developer interactions
//! with the Forge framework.
//!
//! ## Commands
//!
//! ```text
//! forge new <name>        Create a new Forge project
//! forge dev               Start the development server + Forge Studio
//! forge build             Compile for all configured targets
//! forge serve             Run the production server binary
//! forge test              Run the test suite
//! forge lint              Run the Forge linter
//! forge publish           Publish a package to the Foundry registry
//! forge install           Install packages from the Foundry registry
//! forge update            Update installed packages
//! forge init              Initialize Forge in an existing directory
//! ```
//!
//! ## Design
//!
//! The CLI is intentionally thin. Each command delegates immediately to the
//! relevant crate: `forge build` calls into `forge-compiler`, `forge dev`
//! calls into `forge-runtime`, `forge publish` calls into `foundry-client`.
//! No business logic lives in the CLI itself.

use clap::Parser;
use forge_cli::commands::Cli;

#[tokio::main]
async fn main() {
    // Initialize structured logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("forge=info".parse().unwrap()),
        )
        .init();

    let cli = Cli::parse();
    if let Err(e) = cli.run().await {
        eprintln!("error: {}", e);
        std::process::exit(1);
    }
}
