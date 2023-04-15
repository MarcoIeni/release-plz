use release_plz_core::{ReleaseRequest, UpdateRequest};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::PathBuf};
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
            default_update_config.update_changelog = false.into();
        }
        let mut update_request =
            update_request.with_default_package_config(default_update_config.into());
        for (package, config) in self.packages() {
            let mut update_config = config.clone();
            if is_changelog_update_disabled {
                update_config.update.update_changelog = false.into();
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
            default_config.release.no_verify = true;
        }
        if allow_dirty {
            default_config.release.allow_dirty = true;
        }
        let mut release_request =
            release_request.with_default_package_config(default_config.into());

        for (package, config) in self.packages() {
            let mut release_config = config.clone();

            if no_verify {
                release_config.release.release.no_verify = true;
            }
            if allow_dirty {
                release_config.release.release.allow_dirty = true;
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
    /// Configuration applied to all packages by default.
    #[serde(flatten)]
    pub packages_defaults: PackageConfig,
}

/// Configuration for the `update` command.
/// Generical for the whole workspace. Cannot customized on a per-package basic.
#[derive(Serialize, Deserialize, Default, PartialEq, Eq, Debug)]
#[serde(deny_unknown_fields)]
pub struct UpdateConfig {
    /// - If `true`, update all the dependencies in the Cargo.lock file by running `cargo update`.
    /// - If `false`, only update the workspace packages by running `cargo update --workspace`.
    #[serde(default)]
    pub update_dependencies: bool,
    /// Path to the git cliff configuration file. Defaults to the `keep a changelog` configuration.
    #[serde(default)]
    pub changelog_config: Option<PathBuf>,
    /// Allow dirty working directories to be updated. The uncommitted changes will be part of the update.
    #[serde(default)]
    pub allow_dirty: bool,
    /// GitHub/Gitea repository url where your project is hosted.
    /// It is used to generate the changelog release link.
    /// It defaults to the url of the default remote.
    #[serde(default)]
    pub repo_url: Option<Url>,
}

/// Config at the `[[package]]` level.
#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Default, Clone)]
#[serde(deny_unknown_fields)]
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
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Default, Clone)]
#[serde(deny_unknown_fields)]
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
        Self::default()
            .with_git_release(release_plz_core::GitReleaseConfig::enabled(
                value.git_release.enable.into(),
            ))
            .with_no_verify(value.release.no_verify)
            .with_allow_dirty(value.release.allow_dirty)
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
            semver_check: config.semver_check.into(),
            update_changelog: config.update_changelog.into(),
        }
    }
}

impl From<PackageSpecificConfig> for release_plz_core::PackageUpdateConfig {
    fn from(config: PackageSpecificConfig) -> Self {
        Self {
            generic: config.update.into(),
            changelog_path: config.changelog_path,
        }
    }
}

/// Customization for the `release-plz update` command.
/// These can be overridden on a per-package basic.
#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Default, Clone)]
#[serde(deny_unknown_fields)]
pub struct PackageUpdateConfig {
    /// Controls when to run cargo-semver-checks.
    #[serde(default)]
    pub semver_check: SemverCheck,
    /// Whether to create/update changelog or not.
    #[serde(default)]
    update_changelog: BoolDefaultingTrue,
}

impl PackageUpdateConfig {
    pub fn update_changelog(&self) -> bool {
        self.update_changelog.into()
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone, Copy)]
struct BoolDefaultingTrue(bool);

impl Default for BoolDefaultingTrue {
    fn default() -> Self {
        Self(true)
    }
}

impl From<BoolDefaultingTrue> for bool {
    fn from(config: BoolDefaultingTrue) -> Self {
        config.0
    }
}

impl From<bool> for BoolDefaultingTrue {
    fn from(config: bool) -> Self {
        Self(config)
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Default, Clone)]
pub struct PackageReleaseConfig {
    /// Configuration for the GitHub/Gitea/GitLab release.
    #[serde(flatten, default)]
    pub git_release: GitReleaseConfig,
    #[serde(flatten, default)]
    pub release: ReleaseConfig,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Default, Clone)]
#[serde(deny_unknown_fields)]
pub struct ReleaseConfig {
    #[serde(default, rename = "publish_allow_dirty")]
    pub allow_dirty: bool,
    #[serde(default, rename = "publish_no_verify")]
    pub no_verify: bool,
}

/// Whether to run cargo-semver-checks or not.
#[derive(Serialize, Deserialize, Default, PartialEq, Eq, Debug, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum SemverCheck {
    /// Run cargo-semver-checks if the package is a library.
    #[default]
    Lib,
    /// Run cargo-semver-checks.
    Yes,
    /// Don't run cargo-semver-checks.
    No,
}

impl From<SemverCheck> for release_plz_core::RunSemverCheck {
    fn from(config: SemverCheck) -> Self {
        match config {
            SemverCheck::Lib => Self::Lib,
            SemverCheck::Yes => Self::Yes,
            SemverCheck::No => Self::No,
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct GitReleaseConfig {
    /// Publish the GitHub/Gitea release for the created git tag.
    #[serde(default, rename = "git_release_enable")]
    enable: BoolDefaultingTrue,
    /// Whether to mark the created release as not ready for production.
    #[serde(default, rename = "git_release_type")]
    pub release_type: ReleaseType,
    /// If true, will not auto-publish the release.
    #[serde(default, rename = "git_release_draft")]
    pub draft: bool,
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
            update_dependencies = false
            changelog_config = "../git-cliff.toml"
            allow_dirty = false
            repo_url = "https://github.com/MarcoIeni/release-plz"
            git_release_enable = true
            git_release_type = "prod"
            git_release_draft = false
        "#;

        let expected_config = Config {
            workspace: Workspace {
                update: UpdateConfig {
                    update_dependencies: false,
                    changelog_config: Some("../git-cliff.toml".into()),
                    allow_dirty: false,
                    repo_url: Some("https://github.com/MarcoIeni/release-plz".parse().unwrap()),
                },
                packages_defaults: PackageConfig {
                    update: PackageUpdateConfig {
                        semver_check: SemverCheck::Lib,
                        update_changelog: true.into(),
                    },
                    release: PackageReleaseConfig {
                        git_release: GitReleaseConfig {
                            enable: true.into(),
                            release_type: ReleaseType::Prod,
                            draft: false,
                        },
                        ..Default::default()
                    },
                },
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
            update_dependencies = false
            changelog_config = "../git-cliff.toml"
            allow_dirty = false
            repo_url = "https://github.com/MarcoIeni/release-plz"
            semver_check = "lib"
            update_changelog = true

            git_release_enable = true
            git_release_type = "prod"
            git_release_draft = false
        "#;

        let expected_config = Config {
            workspace: Workspace {
                update: UpdateConfig {
                    update_dependencies: false,
                    changelog_config: Some("../git-cliff.toml".into()),
                    allow_dirty: false,
                    repo_url: Some("https://github.com/MarcoIeni/release-plz".parse().unwrap()),
                },
                packages_defaults: PackageConfig {
                    update: PackageUpdateConfig {
                        semver_check: SemverCheck::Lib,
                        update_changelog: true.into(),
                    },
                    release: PackageReleaseConfig {
                        git_release: GitReleaseConfig {
                            enable: true.into(),
                            release_type: ReleaseType::Prod,
                            draft: false,
                        },
                        release: ReleaseConfig {
                            allow_dirty: false,
                            no_verify: false,
                        },
                    },
                },
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
                    update_dependencies: false,
                    changelog_config: Some("../git-cliff.toml".into()),
                    allow_dirty: false,
                    repo_url: Some("https://github.com/MarcoIeni/release-plz".parse().unwrap()),
                },
                packages_defaults: PackageConfig {
                    update: PackageUpdateConfig {
                        semver_check: SemverCheck::Lib,
                        update_changelog: true.into(),
                    },
                    release: PackageReleaseConfig {
                        git_release: GitReleaseConfig {
                            enable: true.into(),
                            release_type: ReleaseType::Prod,
                            draft: false,
                        },
                        ..Default::default()
                    },
                },
            },
            package: [PackageSpecificConfigWithName {
                name: "crate1".to_string(),
                config: PackageSpecificConfig {
                    update: PackageUpdateConfig {
                        semver_check: SemverCheck::No,
                        update_changelog: true.into(),
                    },
                    release: PackageReleaseConfig {
                        git_release: GitReleaseConfig {
                            enable: true.into(),
                            release_type: ReleaseType::Prod,
                            draft: false,
                        },
                        ..Default::default()
                    },
                    changelog_path: Some("./CHANGELOG.md".into()),
                },
            }]
            .into(),
        };

        expect_test::expect![[r#"
            [workspace]
            update_dependencies = false
            changelog_config = "../git-cliff.toml"
            allow_dirty = false
            repo_url = "https://github.com/MarcoIeni/release-plz"
            semver_check = "lib"
            update_changelog = true
            git_release_enable = true
            git_release_type = "prod"
            git_release_draft = false
            publish_allow_dirty = false
            publish_no_verify = false

            [[package]]
            name = "crate1"
            semver_check = "no"
            update_changelog = true
            git_release_enable = true
            git_release_type = "prod"
            git_release_draft = false
            publish_allow_dirty = false
            publish_no_verify = false
            changelog_path = "./CHANGELOG.md"
        "#]]
        .assert_eq(&toml::to_string(&config).unwrap());
    }
}
