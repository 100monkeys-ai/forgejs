//! Forge project scaffolding: generate the output project directory structure.
//!
//! Takes the converted source files and migration metadata and produces a
//! complete Forge project with `forge.toml`, appropriate subdirectories,
//! and all converted `.fx` files placed in their correct locations.

use std::collections::HashSet;

use anyhow::{Context, Result};
use camino::{Utf8Path, Utf8PathBuf};
use tracing::info;

use super::converter::ConversionResult;
use forge_shared::manifest::MigrationMetadata;

/// The result of scaffolding a Forge project.
#[derive(Debug, Clone)]
pub struct ScaffoldResult {
    /// All files written to the output directory.
    pub files_written: Vec<Utf8PathBuf>,
}

/// Generate the Forge project structure at `output_dir`.
///
/// Creates `forge.toml` with project metadata, sets up the directory layout
/// (`app/`, `server/` if applicable), and writes all converted `.fx` files
/// into the appropriate locations.
pub fn scaffold_project(
    output_dir: &Utf8Path,
    converted: &ConversionResult,
    metadata: &MigrationMetadata,
) -> Result<ScaffoldResult> {
    let mut files_written = Vec::new();

    // Create the output directory
    std::fs::create_dir_all(output_dir.as_std_path())
        .with_context(|| format!("failed to create output directory: {output_dir}"))?;

    // Determine project characteristics from metadata
    let has_server = metadata.source_framework == "express"
        || metadata.source_framework == "next"
        || converted.files.iter().any(|f| {
            f.content.contains("server async function") || f.content.contains("forge:router")
        });

    let has_client = metadata.source_framework == "react"
        || metadata.source_framework == "next"
        || converted
            .files
            .iter()
            .any(|f| f.content.contains("$signal") || f.content.contains("$effect"));

    // Create subdirectories
    let app_dir = output_dir.join("app");
    std::fs::create_dir_all(app_dir.as_std_path())?;

    if has_server {
        let server_dir = output_dir.join("server");
        std::fs::create_dir_all(server_dir.as_std_path())?;
    }

    // Generate forge.toml
    let forge_toml = generate_forge_toml(metadata, has_server, has_client);
    let forge_toml_path = output_dir.join("forge.toml");
    std::fs::write(forge_toml_path.as_std_path(), &forge_toml)
        .with_context(|| format!("failed to write {forge_toml_path}"))?;
    files_written.push(forge_toml_path);

    // Write converted source files
    let mut created_dirs: HashSet<Utf8PathBuf> = HashSet::new();

    for file in &converted.files {
        let dest = determine_output_path(output_dir, &file.path, has_server);

        // Ensure parent directory exists
        if let Some(parent) = dest.parent() {
            let parent_buf = Utf8PathBuf::from(parent);
            if !created_dirs.contains(&parent_buf) {
                std::fs::create_dir_all(parent.as_std_path())
                    .with_context(|| format!("failed to create directory: {parent}"))?;
                created_dirs.insert(parent_buf);
            }
        }

        std::fs::write(dest.as_std_path(), &file.content)
            .with_context(|| format!("failed to write {dest}"))?;
        files_written.push(dest);
    }

    info!(files = files_written.len(), output = %output_dir, "project scaffolded");

    Ok(ScaffoldResult { files_written })
}

/// Generate the `forge.toml` manifest content.
fn generate_forge_toml(metadata: &MigrationMetadata, has_server: bool, has_client: bool) -> String {
    let mut toml = String::new();

    // [project]
    toml.push_str("[project]\n");
    toml.push_str(&format!(
        "name = \"migrated-{}\"\n",
        metadata.source_framework
    ));
    toml.push_str("version = \"0.1.0\"\n");
    toml.push_str(&format!(
        "description = \"Migrated from {} application\"\n",
        metadata.source_framework
    ));
    toml.push('\n');

    // [build]
    toml.push_str("[build]\n");
    toml.push_str("entry = \"app/root.fx\"\n");
    toml.push('\n');

    // [[target]] entries
    if has_client && !has_server {
        toml.push_str("[[target]]\n");
        toml.push_str("name = \"production\"\n");
        toml.push_str("type = \"static\"\n");
        toml.push('\n');
    } else if has_server {
        toml.push_str("[[target]]\n");
        toml.push_str("name = \"production\"\n");
        toml.push_str("type = \"server\"\n");
        toml.push('\n');
    }

    // [project.metadata.migration]
    toml.push_str("[project.metadata.migration]\n");
    toml.push_str(&format!(
        "source_framework = \"{}\"\n",
        metadata.source_framework
    ));
    if let Some(ref node_version) = metadata.source_node_version {
        toml.push_str(&format!("source_node_version = \"{node_version}\"\n"));
    }
    toml.push_str(&format!(
        "migration_date = \"{}\"\n",
        metadata.migration_date
    ));
    toml.push_str(&format!(
        "original_dep_count = {}\n",
        metadata.original_dep_count
    ));
    toml.push_str(&format!(
        "resolved_function_count = {}\n",
        metadata.resolved_function_count
    ));
    toml.push_str(&format!(
        "auto_converted_pct = {:.1}\n",
        metadata.auto_converted_pct
    ));

    toml
}

/// Determine where a converted file should be placed in the output project.
///
/// Server-related files go into `server/`, everything else into `app/`.
fn determine_output_path(
    output_dir: &Utf8Path,
    file_path: &Utf8Path,
    has_server: bool,
) -> Utf8PathBuf {
    let file_name = file_path.file_name().unwrap_or("unknown.fx");

    // Heuristic: files that look like server code go into server/
    let is_server_file = has_server
        && (file_name.contains("server")
            || file_name.contains("api")
            || file_name.contains("route")
            || file_name.contains("middleware")
            || file_name.contains("handler"));

    if is_server_file {
        output_dir.join("server").join(file_name)
    } else {
        output_dir.join("app").join(file_name)
    }
}
