//! `forge install [package]` — Install packages from the Foundry registry.

use anyhow::Result;
use clap::Args;

#[derive(Debug, Args)]
pub struct InstallArgs {
    /// Package(s) to install (e.g., "jeshua/my-lib")
    pub packages: Vec<String>,
    /// Install as a dev dependency
    #[arg(long, short = 'D')]
    pub dev: bool,
}

use foundry_client::registry_client::RegistryClient;

pub async fn run(args: InstallArgs) -> Result<()> {
    let client = RegistryClient::new("https://registry.forgejs.com", None);

    for package_spec in args.packages {
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
    }

    Ok(())
}
