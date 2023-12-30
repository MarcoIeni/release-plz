use crate::semver_check::SemverCheck;
use crate::CARGO_TOML;
use crate::{tmp_repo::TempRepo, PackagePath, UpdateRequest, UpdateResult};
use anyhow::Context;
use cargo_metadata::{semver::Version, Package};
use cargo_utils::upgrade_requirement;
use cargo_utils::LocalManifest;
use git_cmd::Repo;
use std::{fs, path::Path};
use tracing::{info, warn};

use tracing::{debug, instrument};

pub type PackagesToUpdate = Vec<(Package, UpdateResult)>;

#[derive(Clone, Default)]
pub struct PackagesUpdate {
    updates: PackagesToUpdate,
    /// New workspace version. If None, the workspace version is not updated.
    /// See cargo [docs](https://doc.rust-lang.org/cargo/reference/workspaces.html#root-package).
    workspace_version: Option<Version>,
}

impl PackagesUpdate {
    pub fn new(updates: PackagesToUpdate) -> Self {
        Self {
            updates,
            workspace_version: None,
        }
    }

    pub fn with_workspace_version(&mut self, workspace_version: Version) {
        self.workspace_version = Some(workspace_version);
    }

    pub fn updates(&self) -> &[(Package, UpdateResult)] {
        &self.updates
    }

    pub fn updates_mut(&mut self) -> &mut PackagesToUpdate {
        &mut self.updates
    }

    pub fn workspace_version(&self) -> Option<&Version> {
        self.workspace_version.as_ref()
    }
}

impl PackagesUpdate {
    pub fn summary(&self) -> String {
        let updates = self.updates_summary();
        let breaking_changes = self.breaking_changes();
        format!("{updates}\n{breaking_changes}")
    }

    fn updates_summary(&self) -> String {
        self.updates
            .iter()
            .map(|(package, update)| {
                if package.version != update.version {
                    format!(
                        "\n* `{}`: {} -> {}{}",
                        package.name,
                        package.version,
                        update.version,
                        update.semver_check.outcome_str()
                    )
                } else {
                    format!("\n* `{}`: {}", package.name, package.version)
                }
            })
            .collect()
    }

    /// Return the list of changes in the changelog of the updated packages
    pub fn changes(&self, project_contains_multiple_pub_packages: bool) -> String {
        self.updates
            .iter()
            .map(|(package, update)| match update.last_changes() {
                Ok(Some(release)) => {
                    let entry_prefix = if project_contains_multiple_pub_packages {
                        format!("## `{}`\n", package.name)
                    } else {
                        "".to_string()
                    };
                    format!(
                        "{}<blockquote>\n\n## {}\n\n{}\n</blockquote>\n\n",
                        entry_prefix,
                        release.title(),
                        release.notes()
                    )
                }
                Ok(None) => {
                    warn!(
                        "no changes detected in changelog of package {}",
                        package.name
                    );
                    "".to_string()
                }
                Err(e) => {
                    warn!(
                        "can't determine changes in changelog of package {}: {e:?}",
                        package.name
                    );
                    "".to_string()
                }
            })
            .collect()
    }

    fn breaking_changes(&self) -> String {
        self.updates
            .iter()
            .map(|(package, update)| match &update.semver_check {
                SemverCheck::Incompatible(incompatibilities) => {
                    format!(
                        "\n### ⚠️ `{}` breaking changes\n\n```{}```\n",
                        package.name, incompatibilities
                    )
                }
                SemverCheck::Compatible | SemverCheck::Skipped => "".to_string(),
            })
            .collect()
    }
}

/// Update a local rust project
#[instrument(skip_all)]
pub fn update(input: &UpdateRequest) -> anyhow::Result<(PackagesUpdate, TempRepo)> {
    let (packages_to_update, repository) =
        crate::next_versions(input).context("failed to determine next versions")?;
    let local_manifest_path = input.local_manifest();
    let all_packages: Vec<Package> =
        cargo_utils::workspace_members(input.cargo_metadata())?.collect();
    update_manifests(&packages_to_update, local_manifest_path, &all_packages)?;
    update_changelogs(input, &packages_to_update)?;
    if !packages_to_update.updates.is_empty() {
        let local_manifest_dir = input.local_manifest_dir()?;
        update_cargo_lock(local_manifest_dir, input.should_update_dependencies())?;

        let there_are_commits_to_push = Repo::new(local_manifest_dir)?.is_clean().is_err();
        if !there_are_commits_to_push {
            info!("the repository is already up-to-date");
        }
    }

    Ok((packages_to_update, repository))
}

fn update_manifests(
    packages_to_update: &PackagesUpdate,
    local_manifest_path: &Path,
    all_packages: &[Package],
) -> anyhow::Result<()> {
    // Distinguish packages type to avoid updating the version of packages that inherit the workspace version
    let (workspace_pkgs, independent_pkgs): (PackagesToUpdate, PackagesToUpdate) =
        packages_to_update
            .updates
            .clone()
            .into_iter()
            .partition(|(p, _)| {
                let local_manifest_path = p.package_path().unwrap().join(CARGO_TOML);
                let local_manifest = LocalManifest::try_new(&local_manifest_path).unwrap();
                local_manifest.version_is_inherited()
            });

    if let Some(new_workspace_version) = packages_to_update.workspace_version() {
        let mut local_manifest = LocalManifest::try_new(local_manifest_path)?;
        local_manifest.set_workspace_version(new_workspace_version);
        local_manifest
            .write()
            .context("can't update workspace version")?;

        for (pkg, _) in workspace_pkgs {
            let package_path = pkg.package_path()?;
            update_dependencies(all_packages, new_workspace_version, package_path)?;
        }
    }

    update_versions(all_packages, &PackagesUpdate::new(independent_pkgs))?;
    Ok(())
}

#[instrument(skip_all)]
fn update_versions(
    all_packages: &[Package],
    packages_to_update: &PackagesUpdate,
) -> anyhow::Result<()> {
    for (package, update) in &packages_to_update.updates {
        let package_path = package.package_path()?;
        set_version(all_packages, package_path, &update.version)?;
    }
    Ok(())
}

#[instrument(skip_all)]
fn update_changelogs(
    update_request: &UpdateRequest,
    local_packages: &PackagesUpdate,
) -> anyhow::Result<()> {
    for (package, update) in &local_packages.updates {
        if let Some(changelog) = update.changelog.as_ref() {
            let changelog_path = update_request.changelog_path(package);
            fs::write(&changelog_path, changelog)
                .with_context(|| format!("cannot write changelog to {:?}", &changelog_path))?;
        }
    }
    Ok(())
}

#[instrument(skip_all)]
fn update_cargo_lock(root: &Path, update_all_dependencies: bool) -> anyhow::Result<()> {
    let mut args = vec!["update"];
    if !update_all_dependencies {
        args.push("--workspace")
    }
    crate::cargo::run_cargo(root, &args)
        .context("error while running cargo to update the Cargo.lock file")?;
    Ok(())
}

#[instrument]
fn set_version(
    all_packages: &[Package],
    package_path: &Path,
    version: &Version,
) -> anyhow::Result<()> {
    debug!("updating version");
    let mut local_manifest =
        LocalManifest::try_new(&package_path.join("Cargo.toml")).context("cannot read manifest")?;
    local_manifest.set_package_version(version);
    local_manifest
        .write()
        .with_context(|| format!("cannot update manifest {:?}", &local_manifest.path))?;

    let package_path = fs::canonicalize(crate::manifest_dir(&local_manifest.path)?)?;
    update_dependencies(all_packages, version, &package_path)?;
    Ok(())
}

/// Update the package version in the dependencies of the other packages.
/// E.g. from:
///
/// ```toml
/// [dependencies]
/// pkg1 = { path = "../pkg1", version = "1.2.3" }
/// ```
///
/// to:

/// ```toml
/// [dependencies]
/// pkg1 = { path = "../pkg1", version = "1.2.4" }
/// ```
///
fn update_dependencies(
    all_packages: &[Package],
    version: &Version,
    package_path: &Path,
) -> anyhow::Result<()> {
    for member in all_packages {
        let mut member_manifest = LocalManifest::try_new(member.manifest_path.as_std_path())?;
        let member_dir = crate::manifest_dir(&member_manifest.path)?.to_owned();
        let deps_to_update = member_manifest
            .get_dependency_tables_mut()
            .flat_map(|t| t.iter_mut().filter_map(|(_, d)| d.as_table_like_mut()))
            .filter(|d| d.contains_key("version"))
            .filter(|d| {
                let dependency_path = d
                    .get("path")
                    .and_then(|i| i.as_str())
                    .and_then(|relpath| fs::canonicalize(member_dir.join(relpath)).ok());
                match dependency_path {
                    Some(dep_path) => dep_path == package_path,
                    None => false,
                }
            });

        for dep in deps_to_update {
            let old_req = dep
                .get("version")
                .expect("filter ensures this")
                .as_str()
                .unwrap_or("*");
            if let Some(new_req) = upgrade_requirement(old_req, version)? {
                dep.insert("version", toml_edit::value(new_req));
            }
        }
        member_manifest.write()?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn changelog_is_printed_correctly_in_workspace() {
        test_logs::init();
        let changelog = r#"
# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [1.1.1] - 2015-05-15

### Fixed
- myfix

### Other
- simple update

## [1.1.0] - 1970-01-01

### fix bugs
- my awesomefix

### other
- complex update
        "#
        .to_string();
        let pkgs = PackagesUpdate::new(vec![
            (
                fake_package::FakePackage::new("foo").into(),
                UpdateResult {
                    version: Version::parse("0.2.0").unwrap(),
                    changelog: Some(changelog.clone()),
                    semver_check: SemverCheck::Compatible,
                },
            ),
            (
                fake_package::FakePackage::new("bar").into(),
                UpdateResult {
                    version: Version::parse("0.2.0").unwrap(),
                    changelog: Some(changelog),
                    semver_check: SemverCheck::Compatible,
                },
            ),
        ]);
        expect_test::expect![[r#"
            ## `foo`
            <blockquote>

            ## [1.1.1] - 2015-05-15

            ### Fixed
            - myfix

            ### Other
            - simple update
            </blockquote>

            ## `bar`
            <blockquote>

            ## [1.1.1] - 2015-05-15

            ### Fixed
            - myfix

            ### Other
            - simple update
            </blockquote>

        "#]]
        .assert_eq(&pkgs.changes(true));
    }

    #[test]
    fn changelog_is_printed_correctly() {
        test_logs::init();
        let changelog = r#"
# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [1.1.1] - 2015-05-15

### Fixed
- myfix

### Other
- simple update

## [1.1.0] - 1970-01-01

### fix bugs
- my awesomefix

### other
- complex update
        "#
        .to_string();
        let pkgs = PackagesUpdate::new(vec![(
            fake_package::FakePackage::new("foo").into(),
            UpdateResult {
                version: Version::parse("0.2.0").unwrap(),
                changelog: Some(changelog),
                semver_check: SemverCheck::Compatible,
            },
        )]);
        expect_test::expect![[r#"
            <blockquote>

            ## [1.1.1] - 2015-05-15

            ### Fixed
            - myfix

            ### Other
            - simple update
            </blockquote>

        "#]]
        .assert_eq(&pkgs.changes(false));
    }
}
