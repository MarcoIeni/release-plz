use std::path::PathBuf;

use tracing::instrument;

use crate::{public_packages, release_order::release_order};

#[derive(Debug)]
pub struct ReleaseRequest {
    /// The manifest of the project you want to release.
    pub local_manifest: PathBuf,
    /// Registry where the packages are published.
    /// The registry name needs to be present in the Cargo config.
    /// If unspecified, crates.io is used.
    pub registry: Option<String>,
    /// Token used to publish to the cargo registry
    pub token: Option<String>,
}

/// Open a pull request with the next packages versions of a local rust project
#[instrument]
pub async fn release(input: &ReleaseRequest) -> anyhow::Result<()> {
    let public_packages = public_packages(&input.local_manifest)?;
    let pkgs = &public_packages.iter().collect::<Vec<_>>();
    let release_order = release_order(pkgs);
    for _package in release_order {
        // TODO publish
    }
    Ok(())
}
