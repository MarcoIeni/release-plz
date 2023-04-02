use release_plz_core::UpdateRequest;
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
    pub package: HashMap<String, PackageSpecificConfig>,
}

impl Config {
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
        for (package, config) in &self.package {
            let mut update_config = config.clone();
            if is_changelog_update_disabled {
                update_config.update.update_changelog = false.into();
            }
            update_request = update_request.with_package_config(package, update_config.into());
        }
        update_request
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

/// Config at the `[package]` level.
#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Default, Clone)]
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

impl From<PackageSpecificConfig> for release_plz_core::PackageReleaseConfig {
    fn from(config: PackageSpecificConfig) -> Self {
        Self {
            generic: release_plz_core::ReleaseConfig {},
            changelog_path: config.changelog_path,
        }
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
#[serde(deny_unknown_fields)]
pub struct PackageReleaseConfig {
    /// Configuration for the GitHub/Gitea/GitLab release.
    #[serde(default)]
    pub git_release: GitReleaseConfig,
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
    #[serde(default)]
    enable: BoolDefaultingTrue,
    /// Whether to mark the created release as not ready for production.
    #[serde(default)]
    pub release_type: ReleaseType,
    /// If true, will not auto-publish the release.
    #[serde(default)]
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

            [workspace.git_release]
            enable = true
            release_type = "prod"
            draft = false
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

            [workspace.git_release]
            enable = true
            release_type = "prod"
            draft = false
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
                    },
                },
            },
            package: [(
                "crate1".to_string(),
                PackageSpecificConfig {
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
                    },
                    changelog_path: None,
                },
            )]
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

            [workspace.git_release]
            enable = true
            release_type = "prod"
            draft = false

            [package.crate1]
            semver_check = "no"
            update_changelog = true

            [package.crate1.git_release]
            enable = true
            release_type = "prod"
            draft = false
        "#]]
        .assert_eq(&toml::to_string(&config).unwrap());
    }
}
