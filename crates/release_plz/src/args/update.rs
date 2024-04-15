use std::path::{Path, PathBuf};

use anyhow::Context;
use cargo_metadata::camino::Utf8Path;
use chrono::NaiveDate;
use clap::builder::{NonEmptyStringValueParser, PathBufValueParser};
use git_cliff_core::config::Config as GitCliffConfig;
use release_plz_core::{fs_utils::to_utf8_path, ChangelogRequest, UpdateRequest};

use crate::config::Config;

use super::repo_command::RepoCommand;

/// Update your project locally, without opening a PR.
/// If `repo_url` contains a GitHub URL, release-plz uses it to add a release
/// link in the changelog.
#[derive(clap::Parser, Debug)]
pub struct Update {
    /// Path to the Cargo.toml of the project you want to update.
    /// If not provided, release-plz will use the Cargo.toml of the current directory.
    /// Both Cargo workspaces and single packages are supported.
    #[arg(long, value_parser = PathBufValueParser::new(), alias = "project-manifest")]
    manifest_path: Option<PathBuf>,
    /// Path to the Cargo.toml contained in the released version of the project you want to update.
    /// If not provided, the packages of your project will be compared with the
    /// ones published in the cargo registry.
    /// Normally, this parameter is used only if the published version of
    /// your project is already available locally.
    /// For example, it could be the path to the project with a `git checkout` on its latest tag.
    /// The git history of this project should be behind the one of the project you want to update.
    #[arg(long, value_parser = PathBufValueParser::new(), alias = "registry-project-manifest")]
    registry_manifest_path: Option<PathBuf>,
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
        conflicts_with("registry_manifest_path"),
        value_parser = NonEmptyStringValueParser::new()
    )]
    registry: Option<String>,
    /// Update all the dependencies in the Cargo.lock file by running `cargo update`.
    /// If this flag is not specified, only update the workspace packages by running `cargo update --workspace`.
    #[arg(short, long)]
    update_deps: bool,
    /// Path to the git-cliff configuration file.
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
    /// It defaults to the url of the default remote.
    #[arg(long, value_parser = NonEmptyStringValueParser::new())]
    repo_url: Option<String>,
    /// Path to the release-plz config file.
    /// Default: `./release-plz.toml`.
    /// If no config file is found, the default configuration is used.
    #[arg(
        long,
        value_name = "PATH",
        value_parser = PathBufValueParser::new()
    )]
    config: Option<PathBuf>,
}

impl RepoCommand for Update {
    fn optional_manifest_path(&self) -> Option<&Utf8Path> {
        self.manifest_path
            .as_deref()
            .map(|p| to_utf8_path(p).unwrap())
    }

    fn repo_url(&self) -> Option<&str> {
        self.repo_url.as_deref()
    }
}

impl Update {
    pub fn config(&self) -> anyhow::Result<Config> {
        super::parse_config(self.config.as_deref())
    }

    fn dependencies_update(&self, config: &Config) -> bool {
        self.update_deps || config.workspace.dependencies_update == Some(true)
    }

    fn allow_dirty(&self, config: &Config) -> bool {
        self.allow_dirty || config.workspace.allow_dirty == Some(true)
    }

    pub fn update_request(
        &self,
        config: Config,
        cargo_metadata: cargo_metadata::Metadata,
    ) -> anyhow::Result<UpdateRequest> {
        let project_manifest = self.manifest_path();
        check_if_cargo_lock_is_ignored(&project_manifest)?;
        let mut update = UpdateRequest::new(cargo_metadata)
            .with_context(|| {
                format!("Cannot find file {project_manifest:?}. Make sure you are inside a rust project or that --manifest-path points to a valid Cargo.toml file.")
            })?
            .with_dependencies_update(self.dependencies_update(&config))
            .with_allow_dirty(self.allow_dirty(&config));
        match self.get_repo_url(&config) {
            Ok(repo_url) => {
                update = update.with_repo_url(repo_url);
            }
            Err(e) => tracing::warn!("Cannot determine repo url. The changelog won't contain the release link. Error: {:?}", e),
        }

        if let Some(registry_manifest_path) = &self.registry_manifest_path {
            let registry_manifest_path = to_utf8_path(registry_manifest_path)?;
            update = update
                .with_registry_manifest_path(registry_manifest_path.to_path_buf())
                .with_context(|| {
                    format!("cannot find project manifest {registry_manifest_path:?}")
                })?;
        }
        update = config.fill_update_config(self.no_changelog, update);
        {
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
                changelog_config: Some(self.changelog_config(&config)?),
            };
            update = update.with_changelog_req(changelog_req);
        }
        if let Some(package) = &self.package {
            update = update.with_single_package(package.clone());
        }
        if let Some(registry) = &self.registry {
            update = update.with_registry(registry.clone());
        }
        if let Some(release_commits) = config.workspace.release_commits {
            update = update.with_release_commits(release_commits.clone())?;
        }

        Ok(update)
    }

    fn changelog_config(&self, config: &Config) -> anyhow::Result<GitCliffConfig> {
        let default_config_path = dirs::config_dir()
            .context("cannot get config dir")?
            .join("git-cliff")
            .join(git_cliff_core::DEFAULT_CONFIG);

        let path = match self.user_changelog_config(config) {
            Some(provided_path) => {
                if provided_path.exists() {
                    provided_path
                } else {
                    anyhow::bail!("cannot read {:?}", provided_path)
                }
            }
            None => &default_config_path,
        };

        // Parse the configuration file.
        let changelog_config = if path.exists() {
            anyhow::ensure!(config.changelog.is_default(), "specifying the `[changelog]` configuration has no effect if `changelog_config` path is specified");
            GitCliffConfig::parse(path).context("failed to parse git-cliff config file")?
        } else {
            config
                .changelog
                .clone()
                .try_into()
                .context("invalid `[changelog] config")?
        };

        Ok(changelog_config)
    }

    /// Changelog configuration specified by user
    fn user_changelog_config<'a>(&'a self, config: &'a Config) -> Option<&'a Path> {
        self.changelog_config
            .as_deref()
            .or(config.workspace.changelog_config.as_deref())
    }
}

fn check_if_cargo_lock_is_ignored(local_manifest: &Utf8Path) -> anyhow::Result<()> {
    let repo_path = release_plz_core::root_repo_path(local_manifest)?;
    let cargo_lock_path = local_manifest.with_file_name("Cargo.lock");
    let is_cargo_lock_ignored = git_cmd::is_file_ignored(&repo_path, &cargo_lock_path);
    anyhow::ensure!(
        !(is_cargo_lock_ignored && cargo_lock_path.exists()),
        "Cargo.lock is present in your .gitignore and is also committed. Remove it from your repository or from your `.gitignore` file."
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use fake_package::metadata::fake_metadata;

    use super::*;

    #[test]
    fn input_generates_correct_release_request() {
        let update_args = Update {
            manifest_path: None,
            registry_manifest_path: None,
            package: None,
            no_changelog: false,
            release_date: None,
            registry: None,
            update_deps: false,
            changelog_config: None,
            allow_dirty: false,
            repo_url: None,
            config: None,
        };
        let config: Config = toml::from_str("").unwrap();
        let req = update_args.update_request(config, fake_metadata()).unwrap();
        let pkg_config = req.get_package_config("aaa");
        assert_eq!(pkg_config, release_plz_core::PackageUpdateConfig::default());
    }
}
