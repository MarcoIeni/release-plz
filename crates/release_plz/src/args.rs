use std::path::PathBuf;

use anyhow::{anyhow, Context};
use chrono::{Date, NaiveDate, Utc};
use release_plz_core::{ChangelogRequest, GitHub, UpdateRequest, CARGO_TOML};
use secrecy::SecretString;
use url::Url;

#[derive(clap::Parser, Debug)]
#[clap(about, version, author)]
pub struct CliArgs {
    #[clap(subcommand)]
    pub command: Command,
}

#[derive(clap::Subcommand, Debug)]
pub enum Command {
    /// Update packages version based on commit messages.
    Update(Update),
    /// Create a Pull Request representing the next release.
    /// The Pull request contains updated packages version and changelog based on commit messages.
    /// Close old PRs opened by release-plz, too.
    ReleasePr(ReleasePr),
}

#[derive(clap::Parser, Debug)]
pub struct Update {
    /// Path to the Cargo.toml of the project you want to update.
    /// If not provided, release-plz will use the Cargo.toml of the current directory.
    /// Both Cargo workspaces and single packages are supported.
    #[clap(long)]
    project_manifest: Option<PathBuf>,
    /// Path to the Cargo.toml contained in the released version of the project you want to update.
    /// If not provided, the packages of your project will be compared with the
    /// ones published in the cargo registry.
    /// Normally, this parameter is used only if the published version of
    /// your project is already available locally.
    /// For example, it could be the path to the project with a `git checkout` on its latest tag.
    /// The git history of this project should be behind the one of the project you want to update.
    #[clap(long)]
    registry_project_manifest: Option<PathBuf>,
    /// Package to update. Use it when you want to update a single package rather than all the
    /// packages contained in the workspace.
    #[clap(short, long)]
    package: Option<String>,
    /// Don't create changelog.
    #[clap(long, conflicts_with("release-date"))]
    no_changelog: bool,
    /// Date of the release. Format: %Y-%m-%d. It defaults to current Utc date.
    #[clap(long, conflicts_with("no-changelog"))]
    release_date: Option<String>,
    /// Registry where the packages are stored.
    /// The registry name needs to be present in the Cargo config.
    /// If unspecified, crates.io is used.
    #[clap(long, conflicts_with("registry-project-manifest"))]
    registry: Option<String>,
}

#[derive(clap::Parser, Debug)]
pub struct ReleasePr {
    #[clap(flatten)]
    pub update: Update,
    /// GitHub token used to create the pull request.
    #[clap(long)]
    pub github_token: SecretString,
    /// GitHub repository url where your project is hosted.
    #[clap(long)]
    pub repo_url: Url,
}

impl Update {
    pub fn update_request(&self) -> anyhow::Result<UpdateRequest> {
        let mut update = UpdateRequest::new(self.local_manifest()).with_context(|| {
            format!("cannot find {CARGO_TOML} file. Make sure you are inside a rust project")
        })?;
        if let Some(registry_project_manifest) = &self.registry_project_manifest {
            update = update
                .with_registry_project_manifest(registry_project_manifest.clone())
                .with_context(|| {
                    format!("cannot find project manifest {registry_project_manifest:?}")
                })?;
        }
        if !self.no_changelog {
            let release_date = self
                .release_date
                .as_ref()
                .map(|date| {
                    NaiveDate::parse_from_str(date, "%Y-%m-%d")
                        .context("cannot parse release_date to y-m-d format")
                })
                .transpose()?
                .map(|date| Date::<Utc>::from_utc(date, Utc));
            update = update.with_changelog(ChangelogRequest { release_date });
        }
        if let Some(package) = &self.package {
            update = update.with_single_package(package.clone());
        }
        if let Some(registry) = &self.registry {
            update = update.with_registry(registry.clone());
        }
        Ok(update)
    }

    fn local_manifest(&self) -> PathBuf {
        match &self.project_manifest {
            Some(manifest) => manifest.clone(),
            None => std::env::current_dir()
                .expect("cannot retrieve current directory")
                .join(CARGO_TOML),
        }
    }
}

impl ReleasePr {
    pub fn github(&self) -> anyhow::Result<GitHub> {
        let segments = self
            .repo_url
            .path_segments()
            .map(|c| c.collect::<Vec<_>>())
            .ok_or_else(|| {
                anyhow!(
                    "cannot find github owner and repo from url {}",
                    self.repo_url
                )
            })?;
        let owner = segments
            .get(0)
            .ok_or_else(|| anyhow!("cannot find github owner from url {}", self.repo_url))?
            .to_string();
        let repo = segments
            .get(1)
            .ok_or_else(|| anyhow!("cannot find github repo from url {}", self.repo_url))?
            .to_string();
        Ok(GitHub::new(owner, repo, self.github_token.clone()))
    }
}
