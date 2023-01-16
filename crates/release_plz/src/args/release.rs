use std::path::PathBuf;

use anyhow::Context;
use clap::builder::{NonEmptyStringValueParser, PathBufValueParser};
use release_plz_core::ReleaseRequest;
use secrecy::SecretString;

use super::local_manifest;

#[derive(clap::Parser, Debug)]
pub struct Release {
    /// Path to the Cargo.toml of the project you want to release.
    /// If not provided, release-plz will use the Cargo.toml of the current directory.
    /// Both Cargo workspaces and single packages are supported.
    #[clap(long, value_parser = PathBufValueParser::new())]
    project_manifest: Option<PathBuf>,
    /// Registry where you want to publish the packages.
    /// The registry name needs to be present in the Cargo config.
    /// If unspecified, the `publish` field of the package manifest is used.
    /// If the `publish` field is empty, crates.io is used.
    #[clap(long)]
    registry: Option<String>,
    /// Token used to publish to the cargo registry.
    #[clap(long, value_parser = NonEmptyStringValueParser::new())]
    token: Option<String>,
    /// Perform all checks without uploading.
    #[clap(long)]
    pub dry_run: bool,
    /// Publish GitHub release for the created git tag.
    #[clap(long)]
    pub git_release: bool,
    /// GitHub repository url.
    #[clap(long, value_parser = NonEmptyStringValueParser::new())]
    pub repo_url: Option<String>,
    /// Git token used to publish the GitHub release.
    #[clap(long, value_parser = NonEmptyStringValueParser::new())]
    pub git_token: Option<String>,
}

impl TryFrom<Release> for ReleaseRequest {
    type Error = anyhow::Error;

    fn try_from(r: Release) -> Result<Self, Self::Error> {
        let git_release = if r.git_release {
            let release = release_plz_core::GitRelease {
                git_token: SecretString::from(
                    r.git_token
                        .context("git_token is required for git_release")?,
                ),
            };
            Some(release)
        } else {
            None
        };
        Ok(ReleaseRequest {
            local_manifest: local_manifest(r.project_manifest.as_deref()),
            registry: r.registry,
            token: r.token.map(SecretString::from),
            dry_run: r.dry_run,
            git_release,
            repo_url: r.repo_url,
        })
    }
}
