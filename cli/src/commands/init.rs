//! `forge init` — Initialize Forge in an existing project directory.

use anyhow::Result;
use clap::Args;

#[derive(Debug, Args)]
pub struct InitArgs {
    /// Directory to initialize (default: current directory)
    pub directory: Option<camino::Utf8PathBuf>,
}

pub async fn run(args: InitArgs) -> Result<()> {
    let dir = args.directory.unwrap_or_else(|| camino::Utf8PathBuf::from("."));

    // Ensure the directory exists
    if !dir.exists() {
        std::fs::create_dir_all(dir.as_std_path())?;
    }

    // Determine project name from directory name, default to "forge-project"
    let name = if let Some(file_name) = dir.file_name() {
        file_name.to_string()
    } else if let Ok(canonical) = dir.canonicalize_utf8() {
        canonical.file_name().unwrap_or("forge-project").to_string()
    } else {
        "forge-project".to_string()
    };

    let forge_toml_path = dir.join("forge.toml");

    if forge_toml_path.exists() {
        anyhow::bail!("forge.toml already exists in {}", dir);
    }

    let toml_content = format!(
        r#"[project]
name = "{}"
version = "0.1.0"

[build]
entry = "app/root.fx"
"#,
        name
    );

    std::fs::write(forge_toml_path.as_std_path(), toml_content)?;

    crate::output::success(&format!(
        "Initialized Forge project '{}' in {}",
        name, dir
    ));

    Ok(())
}
