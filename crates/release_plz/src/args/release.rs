use std::path::PathBuf;

use release_plz_core::ReleaseRequest;

use super::local_manifest;

#[derive(clap::Parser, Debug)]
pub struct Release {
    /// Path to the Cargo.toml of the project you want to release.
    /// If not provided, release-plz will use the Cargo.toml of the current directory.
    /// Both Cargo workspaces and single packages are supported.
    #[clap(long, forbid_empty_values(true))]
    project_manifest: Option<PathBuf>,
    /// Registry where you want to publish the packages.
    /// The registry name needs to be present in the Cargo config.
    /// If unspecified, crates.io is used.
    #[clap(long)]
    registry: Option<String>,
    /// Token used to publish to the cargo registry
    #[clap(long, forbid_empty_values(true))]
    token: Option<String>,
    /// Perform all checks without uploading
    #[clap(long)]
    pub dry_run: bool,
}

impl From<Release> for ReleaseRequest {
    fn from(r: Release) -> Self {
        ReleaseRequest {
            local_manifest: local_manifest(r.project_manifest.as_deref()),
            registry: r.registry,
            token: r.token,
            dry_run: r.dry_run,
        }
    }
}
