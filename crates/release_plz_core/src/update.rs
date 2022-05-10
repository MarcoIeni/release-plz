use crate::{tmp_repo::TempRepo, PackagePath, UpdateRequest, UpdateResult};
use anyhow::{anyhow, Context};
use cargo_edit::{upgrade_requirement, LocalManifest};
use cargo_metadata::{Package, Version};
use std::{fs, path::Path};

use tracing::{debug, instrument};

/// Update a local rust project
#[instrument]
pub fn update(input: &UpdateRequest) -> anyhow::Result<(Vec<(Package, UpdateResult)>, TempRepo)> {
    let (packages_to_update, repository) = crate::next_versions(input)?;
    let all_packages =
        cargo_edit::workspace_members(Some(input.local_manifest())).map_err(|e| {
            anyhow!(
                "cannot read workspace members in manifest {:?}: {e}",
                input.local_manifest()
            )
        })?;
    update_versions(&all_packages, &packages_to_update)?;
    update_changelogs(&packages_to_update)?;
    if !packages_to_update.is_empty() {
        let local_manifest_dir = input.local_manifest_dir()?;
        update_cargo_lock(local_manifest_dir, input.should_update_dependencies())?;
    }
    Ok((packages_to_update, repository))
}

#[instrument(skip_all)]
fn update_versions(
    all_packages: &[Package],
    packages_to_update: &[(Package, UpdateResult)],
) -> anyhow::Result<()> {
    for (package, update) in packages_to_update {
        let package_path = package.package_path()?;
        set_version(all_packages, package_path, &update.version)?;
    }
    Ok(())
}

#[instrument(skip_all)]
fn update_changelogs(local_packages: &[(Package, UpdateResult)]) -> anyhow::Result<()> {
    for (package, update) in local_packages {
        if let Some(changelog) = update.changelog.as_ref() {
            let changelog_path = package.changelog_path()?;
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
    local_manifest.write().expect("cannot update manifest");

    let package_path =
        fs::canonicalize(local_manifest.path.parent().context("at least a parent")?)?;
    update_dependencies(all_packages, version, &package_path)?;
    Ok(())
}

/// Update the package version in the dependencies of the other packages.
fn update_dependencies(
    all_packages: &[Package],
    version: &Version,
    package_path: &Path,
) -> anyhow::Result<()> {
    for member in all_packages {
        let mut member_manifest = LocalManifest::try_new(member.manifest_path.as_std_path())?;
        let member_dir = member_manifest
            .path
            .parent()
            .context("at least a parent")?
            .to_owned();
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
