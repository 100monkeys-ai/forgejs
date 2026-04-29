//! `forge update` — Update installed Foundry packages.

use anyhow::Result;
use clap::Args;

#[derive(Debug, Args)]
pub struct UpdateArgs {
    /// Update only a specific package
    pub package: Option<String>,
}

use foundry_client::registry_client::RegistryClient;

pub async fn run(args: UpdateArgs) -> Result<()> {
    let client = RegistryClient::new("https://registry.forgejs.com", None);

    if let Some(package_spec) = args.package {
        // Split specifier into name and version: "author/name@version"
        // If no version is specified, default to "*"
        let (name, version) = if let Some(idx) = package_spec.find('@') {
            (&package_spec[..idx], &package_spec[idx + 1..])
        } else {
            (package_spec.as_str(), "*")
        };

        match client.resolve(name, version).await {
            Ok(resolved) => {
                tracing::info!(
                    "Resolved {}@{} to {}",
                    resolved.name,
                    resolved.version,
                    resolved.integrity
                );
            }
            Err(e) => {
                tracing::error!("Failed to resolve {}@{}: {}", name, version, e);
                return Err(e.into());
            }
        }
    } else {
        tracing::info!("Updating all packages... (Not fully implemented)");
        // In a real implementation we would read foundry.toml and update all deps
    }

    Ok(())
}
