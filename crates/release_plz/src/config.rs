use anyhow::Context;
use release_plz_core::{ReleaseRequest, UpdateRequest};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::PathBuf, time::Duration};
use url::Url;

#[derive(Serialize, Deserialize, Default, PartialEq, Eq, Debug, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// # Workspace
    /// Global configuration. Applied to all packages by default.
    #[serde(default)]
    pub workspace: Workspace,
    /// # Package
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
        let mut default_update_config = self.workspace.packages_defaults.clone();
        if is_changelog_update_disabled {
            default_update_config.update.changelog_update = false.into();
        }
        let mut update_request =
            update_request.with_default_package_config(default_update_config.into());
        for (package, config) in self.packages() {
            let mut update_config = config.clone();
            update_config = update_config.merge(self.workspace.packages_defaults.clone());
            if is_changelog_update_disabled {
                update_config.package_config.update.changelog_update = false.into();
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
        let mut default_config = self.workspace.packages_defaults.clone();
        if no_verify {
            default_config.release.release.no_verify = Some(true);
        }
        if allow_dirty {
            default_config.release.release.allow_dirty = Some(true);
        }
        let mut release_request =
            release_request.with_default_package_config(default_config.into());

        for (package, config) in self.packages() {
            let mut release_config = config.clone();
            release_config = release_config.merge(self.workspace.packages_defaults.clone());

            if no_verify {
                release_config.package_config.release.release.no_verify = Some(true);
            }
            if allow_dirty {
                release_config.package_config.release.release.allow_dirty = Some(true);
            }
            release_request = release_request.with_package_config(package, release_config.into());
        }
        release_request
    }
}

/// Global configuration.
#[derive(Serialize, Deserialize, Default, PartialEq, Eq, Debug, JsonSchema)]
pub struct Workspace {
    /// Configuration for the `release-plz update` command.
    /// These options also affect the `release-plz release-pr` command.
    #[serde(flatten)]
    pub update: UpdateConfig,
    /// # PR Draft
    /// If `true`, the created release PR will be marked as a draft.
    #[serde(default)]
    pub pr_draft: bool,
    /// # PR Labels
    /// Labels to add to the release PR.
    #[serde(default)]
    pub pr_labels: Vec<String>,
    #[serde(flatten)]
    pub common: CommonCmdConfig,
    /// Configuration applied to all packages by default.
    #[serde(flatten)]
    pub packages_defaults: PackageConfig,
    /// # Publish Timeout
    /// Timeout for the publishing process
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

#[derive(Serialize, Deserialize, Default, PartialEq, Eq, Debug, JsonSchema)]
/// Configuration shared among various commands.
pub struct CommonCmdConfig {
    /// # Repo URL
    /// GitHub/Gitea repository url where your project is hosted.
    /// It is used to generate the changelog release link.
    /// It defaults to the url of the default remote.
    pub repo_url: Option<Url>,
}

/// Configuration for the `update` command.
/// Generical for the whole workspace. Cannot be customized on a per-package basic.
#[derive(Serialize, Deserialize, Default, PartialEq, Eq, Debug, JsonSchema)]
pub struct UpdateConfig {
    /// # Dependencies Update
    /// - If `true`, update all the dependencies in the Cargo.lock file by running `cargo update`.
    /// - If `false` or [`Option::None`], only update the workspace packages by running `cargo update --workspace`.
    pub dependencies_update: Option<bool>,
    /// # Changelog Config
    /// Path to the git cliff configuration file. Defaults to the `keep a changelog` configuration.
    pub changelog_config: Option<PathBuf>,
    /// # Allow Dirty
    /// - If `true`, allow dirty working directories to be updated. The uncommitted changes will be part of the update.
    /// - If `false` or [`Option::None`], the command will fail if the working directory is dirty.
    pub allow_dirty: Option<bool>,
}

/// Config at the `[[package]]` level.
#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone, JsonSchema)]
pub struct PackageSpecificConfig {
    #[serde(flatten)]
    package_config: PackageConfig,
    /// # Changelog Path
    /// Normally the changelog is placed in the same directory of the Cargo.toml file.
    /// The user can provide a custom path here.
    /// This changelog_path needs to be propagated to all the commands:
    /// `update`, `release-pr` and `release`.
    changelog_path: Option<PathBuf>,
    /// # Changelog Include
    /// List of package names.
    /// Include the changelogs of these packages in the changelog of the current package.
    changelog_include: Option<Vec<String>>,
}

impl PackageSpecificConfig {
    /// Merge the package-specific configuration with the global configuration.
    pub fn merge(self, default: PackageConfig) -> PackageSpecificConfig {
        PackageSpecificConfig {
            package_config: self.package_config.merge(default),
            changelog_path: self.changelog_path,
            changelog_include: self.changelog_include,
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone, JsonSchema)]
pub struct PackageSpecificConfigWithName {
    pub name: String,
    #[serde(flatten)]
    pub config: PackageSpecificConfig,
}

impl From<PackageSpecificConfig> for release_plz_core::PackageReleaseConfig {
    fn from(config: PackageSpecificConfig) -> Self {
        let generic = config.package_config.into();

        Self {
            generic,
            changelog_path: config.changelog_path,
        }
    }
}

impl From<PackageConfig> for release_plz_core::ReleaseConfig {
    fn from(value: PackageConfig) -> Self {
        let is_publish_enabled = value.release.release.publish != Some(false);
        let is_git_release_enabled = value.release.git_release.enable != Some(false);
        let is_git_release_draft = value.release.git_release.draft == Some(true);
        let is_git_tag_enabled = value.release.git_tag_enable != Some(false);
        let release = value.common.release != Some(false);

        let mut cfg = Self::default()
            .with_publish(release_plz_core::PublishConfig::enabled(is_publish_enabled))
            .with_git_release(
                release_plz_core::GitReleaseConfig::enabled(is_git_release_enabled)
                    .set_draft(is_git_release_draft),
            )
            .with_git_tag(release_plz_core::GitTagConfig::enabled(is_git_tag_enabled))
            .with_release(release);

        if let Some(no_verify) = value.release.release.no_verify {
            cfg = cfg.with_no_verify(no_verify);
        }
        if let Some(allow_dirty) = value.release.release.allow_dirty {
            cfg = cfg.with_allow_dirty(allow_dirty);
        }
        cfg
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Default, Clone, JsonSchema)]
pub struct PackageCommonConfig {
    /// # Release
    /// Used to toggle off the update/release process for a workspace or package.
    pub release: Option<bool>,
}

impl PackageCommonConfig {
    /// Merge the package-specific configuration with the global configuration.
    pub fn merge(self, default: Self) -> Self {
        Self {
            release: self.release.or(default.release),
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Default, Clone, JsonSchema)]
pub struct PackageConfig {
    /// Options for the `release-plz update` command (therefore `release-plz release-pr` too).
    #[serde(flatten)]
    update: PackageUpdateConfig,
    /// Options for the `release-plz release` command.
    #[serde(flatten)]
    release: PackageReleaseConfig,
    /// Options shared among `update` `release-pr` and `release` commands.
    #[serde(flatten)]
    common: PackageCommonConfig,
}

impl PackageConfig {
    pub fn merge(self, default: Self) -> Self {
        PackageConfig {
            update: self.update.merge(default.update),
            release: self.release.merge(default.release),
            common: self.common.merge(default.common),
        }
    }
}

impl From<PackageConfig> for release_plz_core::UpdateConfig {
    fn from(config: PackageConfig) -> Self {
        Self {
            semver_check: config.update.semver_check != Some(false),
            changelog_update: config.update.changelog_update != Some(false),
            release: config.common.release != Some(false),
        }
    }
}

impl From<PackageSpecificConfig> for release_plz_core::PackageUpdateConfig {
    fn from(config: PackageSpecificConfig) -> Self {
        Self {
            generic: config.package_config.into(),
            changelog_path: config.changelog_path,
            changelog_include: config.changelog_include.unwrap_or_default(),
        }
    }
}

/// Customization for the `release-plz update` command.
/// These can be overridden on a per-package basic.
#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Default, Clone, JsonSchema)]
pub struct PackageUpdateConfig {
    /// # Semver Check
    /// Controls when to run cargo-semver-checks.
    /// If unspecified, run cargo-semver-checks if the package is a library.
    pub semver_check: Option<bool>,
    /// # Changelog Update
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

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Default, Clone, JsonSchema)]
pub struct PackageReleaseConfig {
    /// Configuration for the GitHub/Gitea/GitLab release.
    #[serde(flatten, default)]
    pub git_release: GitReleaseConfig,
    /// # Git Tag Enable
    /// Publish the git tag for the new package version.
    /// Enabled by default.
    pub git_tag_enable: Option<bool>,
    #[serde(flatten, default)]
    pub release: ReleaseConfig,
}

impl PackageReleaseConfig {
    /// Merge the package-specific configuration with the global configuration.
    pub fn merge(self, default: Self) -> Self {
        Self {
            git_release: self.git_release.merge(default.git_release),
            release: self.release.merge(default.release),
            git_tag_enable: self.git_tag_enable.or(default.git_tag_enable),
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Default, Clone, JsonSchema)]
pub struct ReleaseConfig {
    /// # Publish
    /// If `Some(false)`, don't run `cargo publish`.
    pub publish: Option<bool>,
    /// # Publish Allow Dirty
    /// If `Some(true)`, add the `--allow-dirty` flag to the `cargo publish` command.
    #[serde(rename = "publish_allow_dirty")]
    pub allow_dirty: Option<bool>,
    /// # Publish No Verify
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

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone, Default, JsonSchema)]
pub struct GitReleaseConfig {
    /// # Git Release Enable
    /// Publish the GitHub/Gitea release for the created git tag.
    /// Enabled by default.
    #[serde(rename = "git_release_enable")]
    enable: Option<bool>,
    /// # Git Release Type
    /// Whether to mark the created release as not ready for production.
    #[serde(rename = "git_release_type")]
    pub release_type: Option<ReleaseType>,
    /// # Git Release Draft
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

#[derive(Serialize, Deserialize, Default, PartialEq, Eq, Debug, Clone, Copy, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ReleaseType {
    /// # Prod
    /// Will mark the release as ready for production.
    #[default]
    Prod,
    /// # Pre
    /// Will mark the release as not ready for production.
    /// I.e. as pre-release.
    Pre,
    /// # Auto
    /// Will mark the release as not ready for production
    /// in case there is a semver pre-release in the tag e.g. v1.0.0-rc1.
    /// Otherwise, will mark the release as ready for production.
    Auto,
}

#[cfg(test)]
mod tests {
    use super::*;

    const BASE_WORKSPACE_CONFIG: &str = r#"
        [workspace]
        dependencies_update = false
        allow_dirty = false
        changelog_config = "../git-cliff.toml"
        repo_url = "https://github.com/MarcoIeni/release-plz"
        git_release_enable = true
        git_release_type = "prod"
        git_release_draft = false
        publish_timeout = "5m"
    "#;

    const BASE_PACKAGE_CONFIG: &str = r#"
        [[package]]
        name = "crate1"
    "#;

    fn create_base_workspace_config() -> Config {
        Config {
            workspace: Workspace {
                update: UpdateConfig {
                    dependencies_update: Some(false),
                    changelog_config: Some("../git-cliff.toml".into()),
                    allow_dirty: Some(false),
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
                    common: PackageCommonConfig::default(),
                },
                pr_draft: false,
                pr_labels: vec![],
                publish_timeout: Some("5m".to_string()),
            },
            package: [].into(),
        }
    }

    fn create_base_package_config() -> PackageSpecificConfigWithName {
        PackageSpecificConfigWithName {
            name: "crate1".to_string(),
            config: PackageSpecificConfig {
                package_config: PackageConfig {
                    update: PackageUpdateConfig {
                        semver_check: None,
                        changelog_update: None,
                    },
                    release: PackageReleaseConfig {
                        git_release: GitReleaseConfig {
                            enable: None,
                            release_type: None,
                            draft: None,
                        },
                        ..Default::default()
                    },
                    common: PackageCommonConfig::default(),
                },
                changelog_path: None,
                changelog_include: None,
            },
        }
    }

    #[test]
    fn config_without_update_config_is_deserialized() {
        let expected_config = create_base_workspace_config();

        let config: Config = toml::from_str(BASE_WORKSPACE_CONFIG).unwrap();
        assert_eq!(config, expected_config)
    }

    #[test]
    fn config_is_deserialized() {
        let config = &format!(
            "{}\
            changelog_update = true",
            BASE_WORKSPACE_CONFIG
        );

        let mut expected_config = create_base_workspace_config();
        expected_config
            .workspace
            .packages_defaults
            .update
            .changelog_update = true.into();

        let config: Config = toml::from_str(config).unwrap();
        assert_eq!(config, expected_config)
    }

    fn config_package_release_is_deserialized(config_flag: &str, expected_value: bool) {
        let config = &format!(
            "{}\n{}\
            release = {}",
            BASE_WORKSPACE_CONFIG, BASE_PACKAGE_CONFIG, config_flag
        );

        let mut expected_config = create_base_workspace_config();
        let mut package_config = create_base_package_config();
        package_config.config.package_config.common.release = expected_value.into();
        expected_config.package = [package_config].into();

        let config: Config = toml::from_str(config).unwrap();
        assert_eq!(config, expected_config)
    }

    #[test]
    fn config_package_release_is_deserialized_true() {
        config_package_release_is_deserialized("true", true);
    }

    #[test]
    fn config_package_release_is_deserialized_false() {
        config_package_release_is_deserialized("false", false);
    }

    fn config_workspace_release_is_deserialized(config_flag: &str, expected_value: bool) {
        let config = &format!(
            "{}\
            release = {}",
            BASE_WORKSPACE_CONFIG, config_flag
        );

        let mut expected_config = create_base_workspace_config();
        expected_config.workspace.packages_defaults.common.release = expected_value.into();

        let config: Config = toml::from_str(config).unwrap();
        assert_eq!(config, expected_config)
    }

    #[test]
    fn config_workspace_release_is_deserialized_true() {
        config_workspace_release_is_deserialized("true", true);
    }

    #[test]
    fn config_workspace_release_is_deserialized_false() {
        config_workspace_release_is_deserialized("false", false);
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
                pr_draft: false,
                pr_labels: vec!["label1".to_string()],
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
                    common: PackageCommonConfig {
                        release: Some(true),
                    },
                },
                publish_timeout: Some("10m".to_string()),
            },
            package: [PackageSpecificConfigWithName {
                name: "crate1".to_string(),
                config: PackageSpecificConfig {
                    package_config: PackageConfig {
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
                        common: PackageCommonConfig {
                            release: Some(false),
                        },
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
            release = true
            publish_timeout = "10m"

            [[package]]
            name = "crate1"
            semver_check = false
            changelog_update = true
            git_release_enable = true
            git_release_type = "prod"
            git_release_draft = false
            release = false
            changelog_path = "./CHANGELOG.md"
            changelog_include = ["pkg1"]
        "#]]
        .assert_eq(&toml::to_string(&config).unwrap());
    }
}
