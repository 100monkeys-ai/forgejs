use crate::error::FoundryError;
use crate::manifest::foundry_toml::parse_foundry_toml;
use crate::registry_client::RegistryClient;
use flate2::write::GzEncoder;
use flate2::Compression;
use std::fs;
use tracing::info;

pub struct PublishOptions {
    pub dir: camino::Utf8PathBuf,
    pub dry_run: bool,
    pub registry_url: String,
    pub auth_token: Option<String>,
}

pub async fn publish_package(options: PublishOptions) -> Result<(), FoundryError> {
    let manifest_path = options.dir.join("foundry.toml");
    let manifest = parse_foundry_toml(&manifest_path)?;

    let name_parts: Vec<&str> = manifest.package.name.split('/').collect();
    if name_parts.len() != 2 {
        return Err(FoundryError::ManifestParse {
            path: manifest_path.to_string(),
            message: "package name must be in the format 'author/name'".to_string(),
        });
    }
    let author = name_parts[0];
    let name = name_parts[1];

    info!(
        "Publishing {}@{}...",
        manifest.package.name, manifest.package.version
    );

    if options.dry_run {
        info!("Dry run complete. No package published.");
        return Ok(());
    }

    let auth_token = options.auth_token.ok_or(FoundryError::AuthRequired)?;

    let mut tar_gz = Vec::new();
    let enc = GzEncoder::new(&mut tar_gz, Compression::default());
    let mut tar = tar::Builder::new(enc);

    // Pack the directory
    for entry in walkdir::WalkDir::new(&options.dir)
        .into_iter()
        .filter_map(|e: Result<walkdir::DirEntry, walkdir::Error>| e.ok())
    {
        let path = entry.path();
        if path.is_file() {
            let relative_path = path.strip_prefix(&options.dir).unwrap();

            let should_skip = relative_path.components().any(|c| {
                if let std::path::Component::Normal(os_str) = c {
                    let s = os_str.to_string_lossy();
                    s == ".git" || s == ".forge" || s == "node_modules"
                } else {
                    false
                }
            });

            if should_skip {
                continue;
            }

            let mut file = fs::File::open(path)?;
            tar.append_file(relative_path, &mut file)?;
        }
    }
    let enc = tar.into_inner()?;
    enc.finish()?;

    let manifest_content = fs::read_to_string(&manifest_path)?;

    let client = RegistryClient::new(options.registry_url, Some(auth_token));
    client
        .publish(author, name, manifest_content, tar_gz)
        .await?;

    info!(
        "Successfully published {}@{}",
        manifest.package.name, manifest.package.version
    );
    Ok(())
}
