use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::PathBuf};
use url::Url;

#[derive(Serialize, Deserialize, Default, PartialEq, Eq, Debug)]
pub struct Config {
    /// Global configuration. Applied to all packages by default.
    pub workspace: Workspace,
    /// Package specific configuration. This overrides `workspace`.
    /// Not all settings of `workspace` can be overridden.
    #[serde(default)]
    pub package: HashMap<String, PackageConfig>,
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
pub struct UpdateConfig {
    /// - If `true`, update all the dependencies in the Cargo.lock file by running `cargo update`.
    /// - If `false`, only update the workspace packages by running `cargo update --workspace`.
    /// Default: `false`.
    pub update_dependencies: bool,
    /// Path to the git cliff configuration file. Defaults to the `keep a changelog` configuration.
    pub changelog_config: Option<PathBuf>,
    /// Allow dirty working directories to be updated. The uncommitted changes will be part of the update.
    /// Default: `false`.
    pub allow_dirty: bool,
    /// GitHub/Gitea repository url where your project is hosted.
    /// It is used to generate the changelog release link. It defaults to the `origin` url.
    pub repo_url: Option<Url>,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct PackageConfig {
    /// Run cargo-semver-checks.
    pub semver_check: SemverCheck,
    /// Create/update changelog.
    /// Default: `true`.
    pub update_changelog: bool,
    /// Configuration for the GitHub/Gitea/GitLab release.
    pub git_release: GitReleaseConfig,
}

/// Whether to run cargo-semver-checks or not.
#[derive(Serialize, Deserialize, Default, PartialEq, Eq, Debug)]
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

impl Default for PackageConfig {
    fn default() -> Self {
        Self {
            semver_check: SemverCheck::default(),
            update_changelog: true,
            git_release: GitReleaseConfig::default(),
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct GitReleaseConfig {
    /// Publish the GitHub/Gitea release for the created git tag.
    /// Default: `true`
    pub enable: bool,
    /// Whether to mark the created release as not ready for production.
    pub release_type: ReleaseType,
    /// If true, will not auto-publish the release.
    /// Default: `false`.
    pub draft: bool,
}

impl Default for GitReleaseConfig {
    fn default() -> Self {
        Self {
            enable: true,
            release_type: ReleaseType::default(),
            draft: false,
        }
    }
}

#[derive(Serialize, Deserialize, Default, PartialEq, Eq, Debug)]
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
                    semver_check: SemverCheck::Lib,
                    update_changelog: true,
                    git_release: GitReleaseConfig {
                        enable: true,
                        release_type: ReleaseType::Prod,
                        draft: false,
                    },
                },
            },
            package: []
            .into(),
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
                    semver_check: SemverCheck::Lib,
                    update_changelog: true,
                    git_release: GitReleaseConfig {
                        enable: true,
                        release_type: ReleaseType::Prod,
                        draft: false,
                    },
                },
            },
            package: [(
                "crate1".to_string(),
                PackageConfig {
                    semver_check: SemverCheck::No,
                    update_changelog: true,
                    git_release: GitReleaseConfig {
                        enable: true,
                        release_type: ReleaseType::Prod,
                        draft: false,
                    },
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
