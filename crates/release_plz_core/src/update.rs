use crate::{git::Repo, version::NextVersionFromDiff, Diff, LocalPackage};
use anyhow::anyhow;
use cargo_edit::LocalManifest;
use cargo_metadata::{Package, Version};
use folder_compare::FolderCompare;
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};
use tracing::{debug, instrument};

#[derive(Debug)]
pub struct UpdateRequest {
    pub local_manifest: PathBuf,
    pub remote_manifest: Option<PathBuf>,
}

/// Update a local rust project
#[instrument]
pub fn update(input: &UpdateRequest) -> anyhow::Result<(Vec<LocalPackage>, Repo)> {
    let local_crates = list_crates(&input.local_manifest)?;
    let remote_crates = match &input.remote_manifest {
        Some(manifest) => list_crates(manifest)?,
        None => {
            let local_crates_names: Vec<&str> =
                local_crates.iter().map(|c| c.name.as_str()).collect();
            crate::download::download_crates(&local_crates_names)?
        }
    };
    let remote_crates = calculate_remote_crates(remote_crates.into_iter());
    let repository = {
        let mut local_path = input.local_manifest.clone();
        local_path.pop();
        Repo::new(&local_path)?
    };

    debug!("calculating local packages");
    let crates_to_update: Vec<LocalPackage> =
        packages_to_update(local_crates.into_iter(), &remote_crates, &repository)?;
    debug!("local packages calculated");

    if !crates_to_update.is_empty() {
        update_versions(&crates_to_update);
    }
    Ok((crates_to_update, repository))
}

fn get_diff(
    package: &Package,
    remote_crates: &BTreeMap<String, Package>,
    repository: &Repo,
) -> anyhow::Result<Diff> {
    let package_path = package.crate_path();
    debug!("processing local package {}", package.name);
    repository.checkout_head()?;
    let mut diff = Diff::new(false);
    if let Err(_err) = repository.checkout_last_commit_at_path(package_path) {
        // there are no commits for this package
        return Ok(diff);
    }
    loop {
        let current_commit_message = repository.current_commit_message()?;
        if let Some(remote_crate) = remote_crates.get(&package.name) {
            debug!("remote crate {} found", remote_crate.name);
            diff.remote_crate_exists = true;
            let are_packages_equal = {
                let mut remote_path = remote_crate.manifest_path.clone();
                remote_path.pop();
                are_dir_equal(package_path, remote_path.as_ref())
            };
            if are_packages_equal {
                debug!("packages are equal");
                // The local crate is identical to the remote one, which means that
                // the crate was published at this commit, so we will not count this commit
                // as part of the release.
                // We can process the next create.
                break;
            } else if remote_crate.version != package.version {
                debug!("the local package {} has already a different version with respect to the remote package, so release-plz will not update it", package.name);
                break;
            } else {
                debug!("crates are different");
                // At this point of the git history, the two crates are different,
                // which means that this commit is not present in the published package.
                diff.commits.push(current_commit_message.clone());
            }
        } else {
            diff.commits.push(current_commit_message.clone());
        }
        if let Err(_err) = repository.checkout_previous_commit_at_path(package_path) {
            // there are no other commits.
            break;
        }
    }
    Ok(diff)
}

fn are_dir_equal(first: &Path, second: &Path) -> bool {
    let excluded = vec![".git".to_string()];
    let result = FolderCompare::new(first, second, &excluded).unwrap();
    result.changed_files.is_empty() && result.new_files.is_empty()
}

fn packages_to_update(
    crates: impl Iterator<Item = Package>,
    remote_crates: &BTreeMap<String, Package>,
    repository: &Repo,
) -> anyhow::Result<Vec<LocalPackage>> {
    let mut packages_to_update = vec![];
    for c in crates {
        let diff = get_diff(&c, remote_crates, repository)?;
        if diff.should_update_version() {
            packages_to_update.push(LocalPackage { package: c, diff })
        }
    }
    Ok(packages_to_update)
}

trait CratePath {
    fn crate_path(&self) -> &Path;
}

impl CratePath for Package {
    fn crate_path(&self) -> &Path {
        self.manifest_path
            .parent()
            .expect("Cannot find directory containing Cargo.toml file")
            .as_std_path()
    }
}

/// Return [`BTreeMap`] with "package name" as key
fn calculate_remote_crates(crates: impl Iterator<Item = Package>) -> BTreeMap<String, Package> {
    crates
        .map(|c| {
            let package_name = c.name.clone();
            (package_name, c)
        })
        .collect()
}

#[instrument]
fn update_versions(local_crates: &[LocalPackage]) {
    for package in local_crates {
        let current_version = &package.package.version;
        debug!("diff: {:?}", &package.diff);
        let next_version = current_version.next_from_diff(&package.diff);

        debug!("next version: {}", next_version);
        if next_version != *current_version {
            let package_path = package.package.crate_path();
            set_version(package_path, &next_version);
        }
    }
}

#[instrument]
fn set_version(package_path: &Path, version: &Version) {
    debug!("updating version");
    let mut local_manifest =
        LocalManifest::try_new(&package_path.join("Cargo.toml")).expect("cannot read manifest");
    local_manifest.set_package_version(version);
    local_manifest.write().expect("cannot update manifest");
}

fn list_crates(directory: &Path) -> anyhow::Result<Vec<Package>> {
    cargo_edit::workspace_members(Some(directory))
        .map_err(|e| anyhow!("cannot read workspace members: {e}"))
}
