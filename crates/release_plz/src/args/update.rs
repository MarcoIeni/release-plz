use std::path::PathBuf;

use anyhow::Context;
use chrono::NaiveDate;
use clap::builder::{NonEmptyStringValueParser, PathBufValueParser};
use git_cliff_core::config::Config as GitCliffConfig;
use git_cmd::Repo;
use release_plz_core::{ChangelogRequest, RepoUrl, UpdateRequest};

use crate::config::Config;

/// Update your project locally, without opening a PR.
/// If `repo_url` contains a GitHub URL, release-plz uses it to add a release
/// link in the changelog.
#[derive(clap::Parser, Debug)]
pub struct Update {
    /// Path to the Cargo.toml of the project you want to update.
    /// If not provided, release-plz will use the Cargo.toml of the current directory.
    /// Both Cargo workspaces and single packages are supported.
    #[arg(long, value_parser = PathBufValueParser::new())]
    project_manifest: Option<PathBuf>,
    /// Path to the Cargo.toml contained in the released version of the project you want to update.
    /// If not provided, the packages of your project will be compared with the
    /// ones published in the cargo registry.
    /// Normally, this parameter is used only if the published version of
    /// your project is already available locally.
    /// For example, it could be the path to the project with a `git checkout` on its latest tag.
    /// The git history of this project should be behind the one of the project you want to update.
    #[arg(long, value_parser = PathBufValueParser::new())]
    registry_project_manifest: Option<PathBuf>,
    /// Package to update. Use it when you want to update a single package rather than all the
    /// packages contained in the workspace.
    #[arg(
        short,
        long,
        value_parser = NonEmptyStringValueParser::new()
    )]
    package: Option<String>,
    /// Don't create/update changelog.
    #[arg(long, conflicts_with("release_date"))]
    no_changelog: bool,
    /// Date of the release. Format: %Y-%m-%d. It defaults to current Utc date.
    #[arg(
        long,
        conflicts_with("no_changelog"),
        value_parser = NonEmptyStringValueParser::new()
    )]
    release_date: Option<String>,
    /// Registry where the packages are stored.
    /// The registry name needs to be present in the Cargo config.
    /// If unspecified, crates.io is used.
    #[arg(
        long,
        conflicts_with("registry_project_manifest"),
        value_parser = NonEmptyStringValueParser::new()
    )]
    registry: Option<String>,
    /// Update all the dependencies in the Cargo.lock file by running `cargo update`.
    /// If this flag is not specified, only update the workspace packages by running `cargo update --workspace`.
    #[arg(short, long)]
    update_deps: bool,
    /// Path to the git cliff configuration file.
    /// If not provided, `dirs::config_dir()/git-cliff/cliff.toml` is used if present.
    #[arg(
        long,
        env = "GIT_CLIFF_CONFIG",
        value_name = "PATH",
        conflicts_with("no_changelog"),
        value_parser = PathBufValueParser::new()
    )]
    changelog_config: Option<PathBuf>,
    /// Allow dirty working directories to be updated.
    /// The uncommitted changes will be part of the update.
    #[arg(long)]
    allow_dirty: bool,
    /// GitHub/Gitea repository url where your project is hosted.
    /// It is used to generate the changelog release link.
    /// It defaults to the `origin` url.
    #[arg(long, value_parser = NonEmptyStringValueParser::new())]
    repo_url: Option<String>,
}

impl Update {
    pub fn project_manifest(&self) -> PathBuf {
        super::local_manifest(self.project_manifest.as_deref())
    }

    pub fn repo_url(&self) -> anyhow::Result<RepoUrl> {
        match &self.repo_url {
            Some(url) => RepoUrl::new(url),
            None => {
                let project_manifest = self.project_manifest();
                let project_dir = release_plz_core::manifest_dir(&project_manifest)?;
                let repo = Repo::new(project_dir)?;
                RepoUrl::from_repo(&repo)
            }
        }
    }

    fn update_dependencies(&self, config: &Config) -> bool {
        self.update_deps || config.update.update_dependencies
    }

    fn allow_dirty(&self, config: &Config) -> bool {
        self.allow_dirty || config.update.allow_dirty
    }

    pub fn update_request(&self, config: Config) -> anyhow::Result<UpdateRequest> {
        let project_manifest = self.project_manifest();
        let mut update = UpdateRequest::new(project_manifest.clone())
            .with_context(|| {
                format!("Cannot find file {project_manifest:?}. Make sure you are inside a rust project or that --project-manifest points to a valid Cargo.toml file.")
            })?
            .with_update_dependencies(self.update_dependencies(&config))
            .with_allow_dirty(self.allow_dirty(&config));
        match self.repo_url() {
            Ok(repo_url) => {
                update = update.with_repo_url(repo_url);
            }
            Err(e) => tracing::warn!("Cannot determine repo url. The changelog won't contain the release link. Error: {}", e),
        }

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
                .transpose()?;
            let changelog_req = ChangelogRequest {
                release_date,
                changelog_config: self.changelog_config()?,
            };
            update = update.with_changelog(changelog_req);
        }
        if let Some(package) = &self.package {
            update = update.with_single_package(package.clone());
        }
        if let Some(registry) = &self.registry {
            update = update.with_registry(registry.clone());
        }

        Ok(update)
    }

    fn changelog_config(&self) -> anyhow::Result<Option<GitCliffConfig>> {
        let default_config_path = dirs::config_dir()
            .context("cannot get config dir")?
            .join("git-cliff")
            .join(git_cliff_core::DEFAULT_CONFIG);

        let path = match self.changelog_config.clone() {
            Some(provided_path) => {
                if provided_path.exists() {
                    provided_path
                } else {
                    anyhow::bail!("cannot read {:?}", provided_path)
                }
            }
            None => default_config_path,
        };

        // Parse the configuration file.
        let config = if path.exists() {
            Some(GitCliffConfig::parse(&path).context("failed to parse git cliff config file")?)
        } else {
            None
        };

        Ok(config)
    }
}
