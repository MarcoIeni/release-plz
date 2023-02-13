use std::path::PathBuf;

use anyhow::Context;
use clap::builder::{NonEmptyStringValueParser, PathBufValueParser};
use git_cmd::Repo;
use release_plz_core::{GitBackend, GitHub, Gitea, ReleaseRequest, RepoUrl};
use secrecy::SecretString;

use super::{local_manifest, release_pr::GitBackendKind};

#[derive(clap::Parser, Debug, Clone)]
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
    /// Don't verify the contents by building them.
    /// When you pass this flag, `release-plz` adds the `--no-verify` flag to `cargo publish`.
    #[clap(long)]
    pub no_verify: bool,
    /// Allow dirty working directories to be packaged.
    /// When you pass this flag, `release-plz` adds the `--allow-dirty` flag to `cargo publish`.
    #[clap(long)]
    pub allow_dirty: bool,
    /// Publish GitHub release for the created git tag.
    #[clap(long)]
    pub git_release: bool,
    /// GitHub repository url.
    #[clap(long, value_parser = NonEmptyStringValueParser::new())]
    pub repo_url: Option<String>,
    /// Git token used to publish the GitHub release.
    #[clap(long, value_parser = NonEmptyStringValueParser::new())]
    pub git_token: Option<String>,
    /// Kind of git backend
    #[arg(long, value_enum, default_value_t = GitBackendKind::Github)]
    backend: GitBackendKind,
}

impl TryFrom<Release> for ReleaseRequest {
    type Error = anyhow::Error;

    fn try_from(r: Release) -> Result<Self, Self::Error> {
        let git_token = SecretString::from(
            r.clone()
                .git_token
                .context("git_token is required for git_release")?,
        );
        let git_release = if r.git_release {
            let release = release_plz_core::GitRelease {
                git_token: git_token.clone(),
                backend: match r.backend {
                    GitBackendKind::Gitea => {
                        GitBackend::Gitea(Gitea::new(r.repo_url()?, git_token)?)
                    }
                    GitBackendKind::Github => GitBackend::Github(GitHub::new(
                        r.repo_url()?.name,
                        r.repo_url()?.owner,
                        git_token,
                    )),
                },
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
            allow_dirty: r.allow_dirty,
            no_verify: r.no_verify,
        })
    }
}

impl Release {
    pub fn project_manifest(&self) -> PathBuf {
        super::local_manifest(self.project_manifest.as_deref())
    }
    pub fn repo_url(&self) -> anyhow::Result<RepoUrl> {
        match &self.repo_url {
            Some(url) => RepoUrl::new(url.as_str()),
            None => {
                let project_manifest = self.project_manifest();
                let project_dir = project_manifest.parent().context("At least a parent")?;
                let repo = Repo::new(project_dir)?;
                RepoUrl::from_repo(&repo)
            }
        }
    }
}
