use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::PathBuf};
use url::Url;

#[derive(Serialize, Deserialize, Default, PartialEq, Eq, Debug)]
pub struct Config {
    // TODO: can registry be read from Cargo.toml or maybe from cargo config?
    //     /// Registry where the packages are stored. The registry name needs to be present in the Cargo config. If unspecified, crates.io is used.
    //     registry: Option<String>,
    pub update: UpdateConfig,
    /// Configuration applied to all packages by default.
    pub packages_defaults: PackageConfig,
    /// Package specific configuration. This overrides `packages_overrides`.
    pub packages_overrides: HashMap<String, PackageConfig>,
}

/// Configuration for the `update` command.
/// Generical for the whole workspace. Cannot customized on a per-package basic.
///
#[derive(Serialize, Deserialize, Default, PartialEq, Eq, Debug)]
pub struct UpdateConfig {
    /// If `true`, update all the dependencies in the Cargo.lock file by running `cargo update`.
    /// If `false`, only update the workspace packages by running `cargo update --workspace`.
    /// Default: `false`.
    pub update_dependencies: bool,
    /// Path to the git cliff configuration file. Defaults to the `keep a changelog` configuration.
    pub changelog_config: Option<PathBuf>,
    /// Allow dirty working directories to be updated. The uncommitted changes will be part of the update.
    /// Default: `false`.
    pub allow_dirty: bool,
    /// GitHub/Gitea repository url where your project is hosted. It is used to generate the changelog release link. It defaults to the `origin` url.
    pub repo_url: Option<Url>,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct PackageConfig {
    /// Run cargo-semver-checks.
    pub semver_check: SemverCheck,
    /// Create/update changelog.
    /// Default: `true`.
    pub changelog: bool,
    pub release: ReleaseConfig,
}

/// Whether to run cargo-semver-checks or not.
#[derive(Serialize, Deserialize, Default, PartialEq, Eq, Debug)]
#[serde(rename_all = "snake_case")]
pub enum SemverCheck {
    /// Run cargo-semver-checks if the package is a library.
    #[default]
    Libraries,
    /// Run cargo-semver-checks even if the package is a binary.
    True,
    /// Don't run cargo-semver-checks.
    False,
}

impl Default for PackageConfig {
    fn default() -> Self {
        Self {
            semver_check: SemverCheck::default(),
            changelog: true,
            release: ReleaseConfig::default(),
        }
    }
}

/// TODO: allow custom tag names in this struct
#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Default)]
pub struct ReleaseConfig {
    /// Publish the GitHub/Gitea release for the created git tag.
    pub git: GitReleaseConfig,
}

// TODO: custom release name
#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct GitReleaseConfig {
    /// Publish the GitHub/Gitea release for the created git tag.
    /// Default: `true`
    pub enable: bool,
    pub prerelease: Prerelease,
    /// If true, will not auto-publish the release.
    /// Default: `false`.
    pub draft: bool,
}

impl Default for GitReleaseConfig {
    fn default() -> Self {
        Self {
            enable: true,
            prerelease: Prerelease::default(),
            draft: false,
        }
    }
}

#[derive(Serialize, Deserialize, Default, PartialEq, Eq, Debug)]
#[serde(rename_all = "snake_case")]
pub enum Prerelease {
    #[default]
    False,
    /// Will mark the release as not ready for production.
    True,
    /// Will mark the release as not ready for production
    /// in case there is a semver pre-release in the tag e.g. v1.0.0-rc1
    Auto,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_is_deserialized() {
        let config = r#"
            update:
              update_dependencies: false
              changelog_config: ../git-cliff.toml
              allow_dirty: false
              repo_url: https://github.com/MarcoIeni/release-plz
            packages_defaults:
                semver_check: 'true'
                changelog: true
                release:
                    git:
                        enable: true
                        prerelease: false
                        draft: false
            packages_overrides:
                crate1:
                    semver_check: 'true'
                    changelog: true
                    release:
                        git:
                            enable: true
                            prerelease: false
                            draft: false
        "#;

        let expected_config = Config {
            update: UpdateConfig {
                update_dependencies: false,
                changelog_config: Some("../git-cliff.toml".into()),
                allow_dirty: false,
                repo_url: Some("https://github.com/MarcoIeni/release-plz".parse().unwrap()),
            },
            packages_defaults: PackageConfig {
                semver_check: SemverCheck::True,
                changelog: true,
                release: ReleaseConfig {
                    git: GitReleaseConfig {
                        enable: true,
                        prerelease: Prerelease::False,
                        draft: false,
                    },
                },
            },
            packages_overrides: [(
                "crate1".to_string(),
                PackageConfig {
                    semver_check: SemverCheck::True,
                    changelog: true,
                    release: ReleaseConfig {
                        git: GitReleaseConfig {
                            enable: true,
                            prerelease: Prerelease::False,
                            draft: false,
                        },
                    },
                },
            )]
            .into(),
        };

        let config: Config = serde_yaml::from_str(config).unwrap();
        assert_eq!(config, expected_config)
    }

    #[test]
    fn config_is_serialized() {
        let config = Config {
            update: UpdateConfig {
                update_dependencies: false,
                changelog_config: Some("../git-cliff.toml".into()),
                allow_dirty: false,
                repo_url: Some("https://github.com/MarcoIeni/release-plz".parse().unwrap()),
            },
            packages_defaults: PackageConfig {
                semver_check: SemverCheck::True,
                changelog: true,
                release: ReleaseConfig {
                    git: GitReleaseConfig {
                        enable: true,
                        prerelease: Prerelease::False,
                        draft: false,
                    },
                },
            },
            packages_overrides: [(
                "crate1".to_string(),
                PackageConfig {
                    semver_check: SemverCheck::True,
                    changelog: true,
                    release: ReleaseConfig {
                        git: GitReleaseConfig {
                            enable: true,
                            prerelease: Prerelease::False,
                            draft: false,
                        },
                    },
                },
            )]
            .into(),
        };

        expect_test::expect![[r#"
            update:
              update_dependencies: false
              changelog_config: ../git-cliff.toml
              allow_dirty: false
              repo_url: https://github.com/MarcoIeni/release-plz
            packages_defaults:
              semver_check: 'true'
              changelog: true
              release:
                git:
                  enable: true
                  prerelease: 'false'
                  draft: false
            packages_overrides:
              crate1:
                semver_check: 'true'
                changelog: true
                release:
                  git:
                    enable: true
                    prerelease: 'false'
                    draft: false
        "#]]
        .assert_eq(&serde_yaml::to_string(&config).unwrap());
    }
}
