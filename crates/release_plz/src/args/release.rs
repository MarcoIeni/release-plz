use std::path::PathBuf;

use anyhow::Context;
use clap::{
    builder::{NonEmptyStringValueParser, PathBufValueParser},
    ValueEnum,
};
use git_cmd::Repo;
use release_plz_core::{GitBackend, GitHub, GitLab, Gitea, ReleaseRequest, RepoUrl};
use secrecy::SecretString;

use super::local_manifest;

#[derive(clap::Parser, Debug)]
pub struct Release {
    /// Path to the Cargo.toml of the project you want to release.
    /// If not provided, release-plz will use the Cargo.toml of the current directory.
    /// Both Cargo workspaces and single packages are supported.
    #[arg(long, value_parser = PathBufValueParser::new())]
    project_manifest: Option<PathBuf>,
    /// Registry where you want to publish the packages.
    /// The registry name needs to be present in the Cargo config.
    /// If unspecified, the `publish` field of the package manifest is used.
    /// If the `publish` field is empty, crates.io is used.
    #[arg(long)]
    registry: Option<String>,
    /// Token used to publish to the cargo registry.
    #[arg(long, value_parser = NonEmptyStringValueParser::new())]
    token: Option<String>,
    /// Perform all checks without uploading.
    #[arg(long)]
    pub dry_run: bool,
    /// Don't verify the contents by building them.
    /// When you pass this flag, `release-plz` adds the `--no-verify` flag to `cargo publish`.
    #[arg(long)]
    pub no_verify: bool,
    /// Allow dirty working directories to be packaged.
    /// When you pass this flag, `release-plz` adds the `--allow-dirty` flag to `cargo publish`.
    #[arg(long)]
    pub allow_dirty: bool,
    /// Publish GitHub release for the created git tag.
    #[arg(long)]
    pub git_release: bool,
    /// GitHub/Gitea/Gitlab repository url where your project is hosted.
    /// It is used to create the git release.
    /// It defaults to the url of the default remote.
    #[arg(long, value_parser = NonEmptyStringValueParser::new())]
    pub repo_url: Option<String>,
    /// Git token used to publish the GitHub release.
    #[arg(long, value_parser = NonEmptyStringValueParser::new())]
    pub git_token: Option<String>,
    /// Kind of git backend
    #[arg(long, value_enum, default_value_t = ReleaseGitBackendKind::Github)]
    backend: ReleaseGitBackendKind,
}

#[derive(ValueEnum, Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReleaseGitBackendKind {
    #[value(name = "github")]
    Github,
    #[value(name = "gitea")]
    Gitea,
    #[value(name = "gitlab")]
    Gitlab,
}

impl TryFrom<Release> for ReleaseRequest {
    type Error = anyhow::Error;

    fn try_from(r: Release) -> Result<Self, Self::Error> {
        let git_release = if r.git_release {
            let git_token = SecretString::from(
                r.git_token
                    .clone()
                    .context("git_token is required for git_release")?,
            );
            let repo_url = r.repo_url()?;
            let release = release_plz_core::GitRelease {
                backend: match r.backend {
                    ReleaseGitBackendKind::Gitea => {
                        GitBackend::Gitea(Gitea::new(repo_url, git_token)?)
                    }
                    ReleaseGitBackendKind::Github => {
                        GitBackend::Github(GitHub::new(repo_url.owner, repo_url.name, git_token))
                    }
                    ReleaseGitBackendKind::Gitlab => {
                        GitBackend::Gitlab(GitLab::new(repo_url.owner, repo_url.name, git_token))
                    }
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
