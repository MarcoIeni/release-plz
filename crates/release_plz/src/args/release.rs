use std::path::{Path, PathBuf};

use clap::{
    builder::{NonEmptyStringValueParser, PathBufValueParser},
    ValueEnum,
};
use release_plz_core::{GitBackend, GitHub, GitLab, Gitea, ReleaseRequest};
use secrecy::SecretString;

use crate::config::Config;

use super::{local_manifest, repo_command::RepoCommand};

#[derive(clap::Parser, Debug)]
pub struct Release {
    /// Path to the Cargo.toml of the project you want to release.
    /// If not provided, release-plz will use the Cargo.toml of the current directory.
    /// Both Cargo workspaces and single packages are supported.
    #[arg(long, value_parser = PathBufValueParser::new())]
    project_manifest: Option<PathBuf>,
    /// Prevent the packages from being published to a registry.
    /// Publishing will be performed unless this flag is set or the `publish` field of the package manifest is set to `false` or `[]`.
    #[arg(long, value_enum)]
    no_publish: bool,
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
            let repo_url = self.get_repo_url(&config)?;
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
            if self.no_publish {
                anyhow::bail!("Both --no-publish and --registry are set.");
            }
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

        req = config.fill_release_config(self.no_publish, self.allow_dirty, self.no_verify, req);

        Ok(req)
    }
}

impl RepoCommand for Release {
    fn optional_project_manifest(&self) -> Option<&Path> {
        self.project_manifest.as_deref()
    }

    fn repo_url(&self) -> Option<&str> {
        self.repo_url.as_deref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn input_generates_correct_release_request() {
        let config = r#"
            [workspace]
            dependencies_update = false
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
            no_publish: false,
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
