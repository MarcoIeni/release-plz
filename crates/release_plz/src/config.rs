use anyhow::Context;
use release_plz_core::{ReleaseRequest, UpdateRequest};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::PathBuf, time::Duration};
use url::Url;

#[derive(Serialize, Deserialize, Default, PartialEq, Eq, Debug)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// Global configuration. Applied to all packages by default.
    #[serde(default)]
    pub workspace: Workspace,
    /// Package-specific configuration. This overrides `workspace`.
    /// Not all settings of `workspace` can be overridden.
    #[serde(default)]
    package: Vec<PackageSpecificConfigWithName>,
}

impl Config {
    /// Package-specific configurations.
    /// Returns `<package name, package config>`.
    fn packages(&self) -> HashMap<&str, &PackageSpecificConfig> {
        self.package
            .iter()
            .map(|p| (p.name.as_str(), &p.config))
            .collect()
    }

    pub fn fill_update_config(
        &self,
        is_changelog_update_disabled: bool,
        update_request: UpdateRequest,
    ) -> UpdateRequest {
        let mut default_update_config = self.workspace.packages_defaults.update.clone();
        if is_changelog_update_disabled {
            default_update_config.changelog_update = false.into();
        }
        let mut update_request =
            update_request.with_default_package_config(default_update_config.into());
        for (package, config) in self.packages() {
            let mut update_config = config.clone();
            update_config = update_config.merge(self.workspace.packages_defaults.clone());
            if is_changelog_update_disabled {
                update_config.update.changelog_update = false.into();
            }
            update_request = update_request.with_package_config(package, update_config.into());
        }
        update_request
    }

    pub fn fill_release_config(
        &self,
        allow_dirty: bool,
        no_verify: bool,
        release_request: ReleaseRequest,
    ) -> ReleaseRequest {
        let mut default_config = self.workspace.packages_defaults.release.clone();
        if no_verify {
            default_config.release.no_verify = Some(true);
        }
        if allow_dirty {
            default_config.release.allow_dirty = Some(true);
        }
        let mut release_request =
            release_request.with_default_package_config(default_config.into());

        for (package, config) in self.packages() {
            let mut release_config = config.clone();
            release_config = release_config.merge(self.workspace.packages_defaults.clone());

            if no_verify {
                release_config.release.release.no_verify = Some(true);
            }
            if allow_dirty {
                release_config.release.release.allow_dirty = Some(true);
            }
            release_request = release_request.with_package_config(package, release_config.into());
        }
        release_request
    }
}

/// Global configuration.
#[derive(Serialize, Deserialize, Default, PartialEq, Eq, Debug)]
pub struct Workspace {
    /// Configuration for the `release-plz update` command.
    /// These options also affect the `release-plz release-pr` command.
    #[serde(flatten)]
    pub update: UpdateConfig,
    #[serde(flatten)]
    pub release_pr: ReleasePrConfig,
    #[serde(flatten)]
    pub common: CommonCmdConfig,
    /// Configuration applied to all packages by default.
    #[serde(flatten)]
    pub packages_defaults: PackageConfig,
    pub publish_timeout: Option<String>,
}

impl Workspace {
    /// Get the publish timeout. Defaults to 30 minutes.
    pub fn publish_timeout(&self) -> anyhow::Result<Duration> {
        let publish_timeout = self.publish_timeout.as_deref().unwrap_or("30m");
        duration_str::parse(publish_timeout)
            .with_context(|| format!("invalid publish_timeout {}", publish_timeout))
    }
}

#[derive(Serialize, Deserialize, Default, PartialEq, Eq, Debug)]
/// Configuration shared among various commands.
pub struct CommonCmdConfig {
    /// GitHub/Gitea repository url where your project is hosted.
    /// It is used to generate the changelog release link.
    /// It defaults to the url of the default remote.
    pub repo_url: Option<Url>,
}

/// Configuration for the `update` command.
/// Generical for the whole workspace. Cannot be customized on a per-package basic.
#[derive(Serialize, Deserialize, Default, PartialEq, Eq, Debug)]
pub struct UpdateConfig {
    /// - If `true`, update all the dependencies in the Cargo.lock file by running `cargo update`.
    /// - If `false` or [`Option::None`], only update the workspace packages by running `cargo update --workspace`.
    pub dependencies_update: Option<bool>,
    /// Path to the git cliff configuration file. Defaults to the `keep a changelog` configuration.
    pub changelog_config: Option<PathBuf>,
    /// - If `true`, allow dirty working directories to be updated. The uncommitted changes will be part of the update.
    /// - If `false` or [`Option::None`], the command will fail if the working directory is dirty.
    pub allow_dirty: Option<bool>,
}

/// Configuration for the `release-pr` command.
/// Generical for the whole workspace. Cannot be customized on a per-package basic.
#[derive(Serialize, Deserialize, Default, PartialEq, Eq, Debug)]
pub struct ReleasePrConfig {
    /// If `true`, the created release PR will be marked as a draft.
    #[serde(default)]
    pub pr_draft: bool,
    /// Labels to add to the release PR.
    #[serde(default)]
    pub pr_labels: Vec<String>,
}

/// Config at the `[[package]]` level.
#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
pub struct PackageSpecificConfig {
    /// Options for the `release-plz update` command (therefore `release-plz release-pr` too).
    #[serde(flatten)]
    update: PackageUpdateConfig,
    /// Options for the `release-plz release` command.
    #[serde(flatten)]
    release: PackageReleaseConfig,
    /// Normally the changelog is placed in the same directory of the Cargo.toml file.
    /// The user can provide a custom path here.
    /// This changelog_path needs to be propagated to all the commands:
    /// `update`, `release-pr` and `release`.
    changelog_path: Option<PathBuf>,
    /// List of package names.
    /// Include the changelogs of these packages in the changelog of the current package.
    changelog_include: Option<Vec<String>>,
}

impl PackageSpecificConfig {
    /// Merge the package-specific configuration with the global configuration.
    pub fn merge(self, default: PackageConfig) -> PackageSpecificConfig {
        PackageSpecificConfig {
            update: self.update.merge(default.update),
            release: self.release.merge(default.release),
            changelog_path: self.changelog_path,
            changelog_include: self.changelog_include,
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
pub struct PackageSpecificConfigWithName {
    pub name: String,
    #[serde(flatten)]
    pub config: PackageSpecificConfig,
}

impl From<PackageSpecificConfig> for release_plz_core::PackageReleaseConfig {
    fn from(config: PackageSpecificConfig) -> Self {
        let generic = config.release.into();

        Self {
            generic,
            changelog_path: config.changelog_path,
        }
    }
}

impl From<PackageReleaseConfig> for release_plz_core::ReleaseConfig {
    fn from(value: PackageReleaseConfig) -> Self {
        let is_publish_enabled = value.release.publish != Some(false);
        let is_git_release_enabled = value.git_release.enable != Some(false);
        let is_git_release_draft = value.git_release.draft == Some(true);
        let is_git_tag_enabled = value.git_tag.enable != Some(false);
        let mut cfg = Self::default()
            .with_publish(release_plz_core::PublishConfig::enabled(is_publish_enabled))
            .with_git_release(
                release_plz_core::GitReleaseConfig::enabled(is_git_release_enabled)
                    .set_draft(is_git_release_draft),
            )
            .with_git_tag(release_plz_core::GitTagConfig::enabled(is_git_tag_enabled));

        if let Some(no_verify) = value.release.no_verify {
            cfg = cfg.with_no_verify(no_verify);
        }
        if let Some(allow_dirty) = value.release.allow_dirty {
            cfg = cfg.with_allow_dirty(allow_dirty);
        }
        cfg
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Default, Clone)]
pub struct PackageConfig {
    /// Options for the `release-plz update` command (therefore `release-plz release-pr` too).
    #[serde(flatten)]
    update: PackageUpdateConfig,
    /// Options for the `release-plz release` command.
    #[serde(flatten)]
    release: PackageReleaseConfig,
}

impl From<PackageUpdateConfig> for release_plz_core::UpdateConfig {
    fn from(config: PackageUpdateConfig) -> Self {
        Self {
            semver_check: config.semver_check != Some(false),
            changelog_update: config.changelog_update != Some(false),
        }
    }
}

impl From<PackageSpecificConfig> for release_plz_core::PackageUpdateConfig {
    fn from(config: PackageSpecificConfig) -> Self {
        Self {
            generic: config.update.into(),
            changelog_path: config.changelog_path,
            changelog_include: config.changelog_include.unwrap_or_default(),
        }
    }
}

/// Customization for the `release-plz update` command.
/// These can be overridden on a per-package basic.
#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Default, Clone)]
pub struct PackageUpdateConfig {
    /// Controls when to run cargo-semver-checks.
    /// If unspecified, run cargo-semver-checks if the package is a library.
    pub semver_check: Option<bool>,
    /// Whether to create/update changelog or not.
    /// If unspecified, the changelog is updated.
    pub changelog_update: Option<bool>,
}

impl PackageUpdateConfig {
    /// Merge the package-specific configuration with the global configuration.
    pub fn merge(self, default: PackageUpdateConfig) -> PackageUpdateConfig {
        PackageUpdateConfig {
            semver_check: self.semver_check.or(default.semver_check),
            changelog_update: self.changelog_update.or(default.changelog_update),
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Default, Clone)]
pub struct PackageReleaseConfig {
    /// Configuration for the GitHub/Gitea/GitLab release.
    #[serde(flatten, default)]
    pub git_release: GitReleaseConfig,
    #[serde(flatten, default)]
    pub git_tag: GitTagConfig,
    #[serde(flatten, default)]
    pub release: ReleaseConfig,
}

impl PackageReleaseConfig {
    /// Merge the package-specific configuration with the global configuration.
    pub fn merge(self, default: Self) -> Self {
        Self {
            git_release: self.git_release.merge(default.git_release),
            release: self.release.merge(default.release),
            git_tag: self.git_tag.merge(default.git_tag),
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Default, Clone)]
pub struct ReleaseConfig {
    /// If `Some(false)`, don't run `cargo publish`.
    pub publish: Option<bool>,
    /// If `Some(true)`, add the `--allow-dirty` flag to the `cargo publish` command.
    #[serde(rename = "publish_allow_dirty")]
    pub allow_dirty: Option<bool>,
    /// If `Some(true)`, add the `--no-verify` flag to the `cargo publish` command.
    #[serde(rename = "publish_no_verify")]
    pub no_verify: Option<bool>,
}

impl ReleaseConfig {
    /// Merge the package-specific configuration with the global configuration.
    pub fn merge(self, default: Self) -> Self {
        Self {
            publish: self.publish.or(default.publish),
            allow_dirty: self.allow_dirty.or(default.allow_dirty),
            no_verify: self.no_verify.or(default.no_verify),
        }
    }
}

/// Whether to run cargo-semver-checks or not.
/// Note: you can only run cargo-semver-checks on a library.
#[derive(Serialize, Deserialize, Default, PartialEq, Eq, Debug, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum SemverCheck {
    /// Run cargo-semver-checks.
    #[default]
    Yes,
    /// Don't run cargo-semver-checks.
    No,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone, Default)]
pub struct GitTagConfig {
    /// Publish the git tag for the new package version.
    /// Enabled by default.
    #[serde(rename = "git_tag_enable")]
    enable: Option<bool>,
}

impl GitTagConfig {
    pub fn merge(self, default: GitTagConfig) -> Self {
        Self {
            enable: self.enable.or(default.enable),
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone, Default)]
pub struct GitReleaseConfig {
    /// Publish the GitHub/Gitea release for the created git tag.
    /// Enabled by default.
    #[serde(rename = "git_release_enable")]
    enable: Option<bool>,
    /// Whether to mark the created release as not ready for production.
    #[serde(rename = "git_release_type")]
    pub release_type: Option<ReleaseType>,
    /// If true, will not auto-publish the release.
    #[serde(rename = "git_release_draft")]
    pub draft: Option<bool>,
}

impl GitReleaseConfig {
    /// Merge the package-specific configuration with the global configuration.
    pub fn merge(self, default: Self) -> Self {
        Self {
            enable: self.enable.or(default.enable),
            release_type: self.release_type.or(default.release_type),
            draft: self.draft.or(default.draft),
        }
    }
}

#[derive(Serialize, Deserialize, Default, PartialEq, Eq, Debug, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum ReleaseType {
    /// Will mark the release as ready for production.
    #[default]
    Prod,
    /// Will mark the release as not ready for production.
    /// I.e. as pre-release.
    Pre,
    /// Will mark the release as not ready for production
    /// in case there is a semver pre-release in the tag e.g. v1.0.0-rc1.
    /// Otherwise, will mark the release as ready for production.
    Auto,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_without_update_config_is_deserialized() {
        let config = r#"
            [workspace]
            dependencies_update = false
            changelog_config = "../git-cliff.toml"
            repo_url = "https://github.com/MarcoIeni/release-plz"
            git_release_enable = true
            git_release_type = "prod"
            git_release_draft = false
            publish_timeout = "10m"
        "#;

        let expected_config = Config {
            workspace: Workspace {
                update: UpdateConfig {
                    dependencies_update: Some(false),
                    changelog_config: Some("../git-cliff.toml".into()),
                    allow_dirty: None,
                },
                common: CommonCmdConfig {
                    repo_url: Some("https://github.com/MarcoIeni/release-plz".parse().unwrap()),
                },
                packages_defaults: PackageConfig {
                    update: PackageUpdateConfig {
                        semver_check: None,
                        changelog_update: None,
                    },
                    release: PackageReleaseConfig {
                        git_release: GitReleaseConfig {
                            enable: Some(true),
                            release_type: Some(ReleaseType::Prod),
                            draft: Some(false),
                        },
                        ..Default::default()
                    },
                },
                release_pr: ReleasePrConfig {
                    pr_draft: false,
                    pr_labels: vec![],
                },
                publish_timeout: Some("10m".to_string()),
            },
            package: [].into(),
        };

        let config: Config = toml::from_str(config).unwrap();
        assert_eq!(config, expected_config)
    }

    #[test]
    fn config_is_deserialized() {
        let config = r#"
            [workspace]
            changelog_config = "../git-cliff.toml"
            allow_dirty = false
            repo_url = "https://github.com/MarcoIeni/release-plz"
            changelog_update = true

            git_release_enable = true
            git_release_type = "prod"
            git_release_draft = false
            publish_timeout = "5s"
        "#;

        let expected_config = Config {
            workspace: Workspace {
                update: UpdateConfig {
                    dependencies_update: None,
                    changelog_config: Some("../git-cliff.toml".into()),
                    allow_dirty: Some(false),
                },
                common: CommonCmdConfig {
                    repo_url: Some("https://github.com/MarcoIeni/release-plz".parse().unwrap()),
                },
                release_pr: ReleasePrConfig {
                    pr_draft: false,
                    pr_labels: vec![],
                },
                packages_defaults: PackageConfig {
                    update: PackageUpdateConfig {
                        semver_check: None,
                        changelog_update: true.into(),
                    },
                    release: PackageReleaseConfig {
                        git_release: GitReleaseConfig {
                            enable: true.into(),
                            release_type: Some(ReleaseType::Prod),
                            draft: Some(false),
                        },
                        git_tag: GitTagConfig { enable: None },
                        release: ReleaseConfig {
                            publish: None,
                            allow_dirty: None,
                            no_verify: None,
                        },
                    },
                },
                publish_timeout: Some("5s".to_string()),
            },
            package: [].into(),
        };

        let config: Config = toml::from_str(config).unwrap();
        assert_eq!(config, expected_config)
    }

    #[test]
    fn config_is_serialized() {
        let config = Config {
            workspace: Workspace {
                update: UpdateConfig {
                    dependencies_update: None,
                    changelog_config: Some("../git-cliff.toml".into()),
                    allow_dirty: None,
                },
                common: CommonCmdConfig {
                    repo_url: Some("https://github.com/MarcoIeni/release-plz".parse().unwrap()),
                },
                release_pr: ReleasePrConfig {
                    pr_draft: false,
                    pr_labels: vec!["label1".to_string()],
                },
                packages_defaults: PackageConfig {
                    update: PackageUpdateConfig {
                        semver_check: None,
                        changelog_update: true.into(),
                    },
                    release: PackageReleaseConfig {
                        git_release: GitReleaseConfig {
                            enable: true.into(),
                            release_type: Some(ReleaseType::Prod),
                            draft: Some(false),
                        },
                        ..Default::default()
                    },
                },
                publish_timeout: Some("10m".to_string()),
            },
            package: [PackageSpecificConfigWithName {
                name: "crate1".to_string(),
                config: PackageSpecificConfig {
                    update: PackageUpdateConfig {
                        semver_check: Some(false),
                        changelog_update: true.into(),
                    },
                    release: PackageReleaseConfig {
                        git_release: GitReleaseConfig {
                            enable: true.into(),
                            release_type: Some(ReleaseType::Prod),
                            draft: Some(false),
                        },
                        ..Default::default()
                    },
                    changelog_path: Some("./CHANGELOG.md".into()),
                    changelog_include: Some(vec!["pkg1".to_string()]),
                },
            }]
            .into(),
        };

        expect_test::expect![[r#"
            [workspace]
            changelog_config = "../git-cliff.toml"
            pr_draft = false
            pr_labels = ["label1"]
            repo_url = "https://github.com/MarcoIeni/release-plz"
            changelog_update = true
            git_release_enable = true
            git_release_type = "prod"
            git_release_draft = false
            publish_timeout = "10m"

            [[package]]
            name = "crate1"
            semver_check = false
            changelog_update = true
            git_release_enable = true
            git_release_type = "prod"
            git_release_draft = false
            changelog_path = "./CHANGELOG.md"
            changelog_include = ["pkg1"]
        "#]]
        .assert_eq(&toml::to_string(&config).unwrap());
    }
}
