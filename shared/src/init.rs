//! Project initialization utility logic.
//!
//! Handles generating new project structures and configurations.

use camino::Utf8PathBuf;
use std::io;

use crate::manifest::{BuildConfig, ForgeManifest, ProjectMetadata};

/// Parameters for project initialization.
#[derive(Debug, Clone)]
pub struct InitOptions {
    /// Directory to initialize the project in.
    pub target_dir: Utf8PathBuf,
}

/// Errors that can occur during project initialization.
#[derive(Debug, thiserror::Error)]
pub enum InitError {
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),
    #[error("forge.toml already exists in {0}")]
    AlreadyExists(Utf8PathBuf),
    #[error("Failed to serialize forge.toml: {0}")]
    Serialization(#[from] toml::ser::Error),
}

/// Initializes a new Forge project at the specified directory.
///
/// Creates the target directory if it does not exist, determines a sensible
/// project name from the path, and generates a default `forge.toml` manifest.
pub fn init_project(options: InitOptions) -> Result<(Utf8PathBuf, String), InitError> {
    let dir = options.target_dir;

    if !dir.exists() {
        std::fs::create_dir_all(dir.as_std_path())?;
    }

    let forge_toml_path = dir.join("forge.toml");
    if forge_toml_path.exists() {
        return Err(InitError::AlreadyExists(dir));
    }

    // Determine project name from directory name, using canonicalize to resolve "." and ".."
    let name = if let Ok(canonical) = dir.canonicalize_utf8() {
        if let Some(file_name) = canonical.file_name() {
            file_name.to_string()
        } else {
            "forge-project".to_string()
        }
    } else if let Some(file_name) = dir.file_name() {
        if file_name == "." || file_name == ".." || file_name.is_empty() {
            "forge-project".to_string()
        } else {
            file_name.to_string()
        }
    } else {
        "forge-project".to_string()
    };

    let manifest = ForgeManifest {
        project: ProjectMetadata {
            name: name.clone(),
            version: "0.1.0".to_string(),
            description: None,
            authors: vec![],
            license: None,
        },
        build: BuildConfig {
            entry: Some(Utf8PathBuf::from("app/root.fx")),
            output: None,
            source_maps: None,
        },
        dev: Default::default(),
        target: vec![],
        dependencies: Default::default(),
        dev_dependencies: Default::default(),
    };

    let toml_content = toml::to_string(&manifest)?;

    std::fs::write(forge_toml_path.as_std_path(), toml_content)?;

    Ok((dir, name))
}
