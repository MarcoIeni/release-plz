use std::path::PathBuf;

use anyhow::Context;
use clap::{
    builder::{NonEmptyStringValueParser, PathBufValueParser},
    ValueEnum,
};
use git_cmd::Repo;
use release_plz_core::{GitBackend, GitHub, GitLab, Gitea, ReleaseRequest, RepoUrl};
use secrecy::SecretString;

use crate::config::Config;

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

impl Release {
    pub fn release_request(self, config: Config) -> anyhow::Result<ReleaseRequest> {
        let git_release = if let Some(git_token) = &self.git_token {
            let git_token = SecretString::from(git_token.clone());
            let repo_url = self.repo_url()?;
            let release = release_plz_core::GitRelease {
                backend: match self.backend {
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
        let mut req = ReleaseRequest::new(local_manifest(self.project_manifest.as_deref()))
            .with_dry_run(self.dry_run);

        if let Some(registry) = self.registry {
            req = req.with_registry(registry);
        }
        if let Some(token) = self.token {
            req = req.with_token(SecretString::from(token));
        }
        if let Some(repo_url) = self.repo_url {
            req = req.with_repo_url(repo_url);
        }
        if let Some(git_release) = git_release {
            req = req.with_git_release(git_release);
        }

        req = config.fill_release_config(self.allow_dirty, self.no_verify, req);

        Ok(req)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn input_generates_correct_release_request() {
        let config = r#"
            [workspace]
            update_dependencies = false
            changelog_config = "../git-cliff.toml"
            allow_dirty = false
            repo_url = "https://github.com/MarcoIeni/release-plz"
            publish_allow_dirty = true
            git_release_enable = true
            git_release_type = "prod"
            git_release_draft = false
        "#;

        let release_args = default_args();
        let config: Config = toml::from_str(config).unwrap();
        let actual_request = release_args.release_request(config).unwrap();
        assert!(actual_request.allow_dirty("aaa"));
    }

    #[test]
    fn package_config_is_overriden() {
        let config = r#"
            [workspace]
            publish_allow_dirty = false
            publish_no_verify = true

            [[package]]
            name = "aaa"
            publish_allow_dirty = true
        "#;

        let release_args = default_args();
        let config: Config = toml::from_str(config).unwrap();
        let actual_request = release_args.release_request(config).unwrap();
        assert!(actual_request.allow_dirty("aaa"));
        assert!(actual_request.no_verify("aaa"));
    }

    fn default_args() -> Release {
        Release {
            allow_dirty: false,
            no_verify: false,
            project_manifest: None,
            registry: None,
            token: None,
            dry_run: false,
            repo_url: None,
            git_token: None,
            backend: ReleaseGitBackendKind::Github,
        }
    }

    #[test]
    fn default_config_is_converted_to_default_release_request() {
        let release_args = default_args();
        let config: Config = toml::from_str("").unwrap();
        let request = release_args.release_request(config).unwrap();
        let pkg_config = request.get_package_config("aaa");
        let expected = release_plz_core::PackageReleaseConfig {
            generic: release_plz_core::ReleaseConfig::default(),
            changelog_path: None,
        };
        assert_eq!(pkg_config, expected);
        assert!(pkg_config.generic.git_release().is_enabled());
    }
}
