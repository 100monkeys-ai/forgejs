//! `forge build` — Compile the project for deployment targets.

use anyhow::{Context, Result};
use clap::Args;
use std::env;

#[derive(Debug, Args)]
pub struct BuildArgs {
    /// Build a specific named target (default: all targets in forge.toml)
    #[arg(long)]
    pub target: Option<String>,
    /// Enable minification (default: true for release)
    #[arg(long)]
    pub minify: bool,
}

pub async fn run(args: BuildArgs) -> Result<()> {
    let current_dir = camino::Utf8PathBuf::try_from(env::current_dir()?)?;
    let manifest_path = current_dir.join("forge.toml");

    crate::output::info(&format!("Reading manifest at {}", manifest_path));

    let manifest = forge_compiler::parser::forge_toml::parse_forge_toml(&manifest_path)
        .context("Failed to read or parse forge.toml")?;

    let targets_to_build: Vec<_> = if let Some(target_name) = &args.target {
        manifest
            .target
            .into_iter()
            .filter(|t| t.name == *target_name)
            .collect()
    } else {
        manifest.target
    };

    if targets_to_build.is_empty() {
        crate::output::warn("No targets found to build.");
        return Ok(());
    }

    for target in targets_to_build {
        crate::output::info(&format!("Building target '{}'", target.name));

        let options = forge_compiler::CompileOptions {
            target: target.target_type.clone(),
            source_maps: manifest.build.source_maps.unwrap_or(true),
            minify: args.minify,
            project_root: current_dir.clone(),
        };

        match forge_compiler::compile(options) {
            Ok(output) => {
                crate::output::success(&format!(
                    "Successfully built target '{}' ({} assets)",
                    target.name,
                    output.assets.len() + 1
                ));
            }
            Err(e) => {
                crate::output::error(&format!("Failed to build target '{}': {}", target.name, e));
                return Err(e.into());
            }
        }
    }

    crate::output::success("Build completed successfully");

    Ok(())
}
