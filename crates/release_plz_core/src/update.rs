use crate::{git::Repo, version::NextVersionFromDiff, Diff};
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

/// Determine next version of packages
#[instrument]
pub fn next_versions(input: &UpdateRequest) -> anyhow::Result<(Vec<(Package, Version)>, Repo)> {
    let local_crates = list_crates(&input.local_manifest)?;
    let remote_crates = get_remote_crates(input.remote_manifest.as_ref(), &local_crates)?;
    let repository = {
        let mut local_path = input.local_manifest.clone();
        local_path.pop();
        Repo::new(&local_path)?
    };

    debug!("calculating local packages");
    let crates_to_update =
        packages_to_update(local_crates.into_iter(), &remote_crates, &repository)?;
    debug!("local packages calculated");
    Ok((crates_to_update, repository))
}

/// Update a local rust project
#[instrument]
pub fn update(input: &UpdateRequest) -> anyhow::Result<(Vec<(Package, Version)>, Repo)> {
    let (crates_to_update, repository) = next_versions(input)?;

    update_versions(&crates_to_update);
    Ok((crates_to_update, repository))
}

#[instrument(
    skip_all,
    fields(package = %package.name)
)]
fn get_diff(
    package: &Package,
    remote_crates: &BTreeMap<String, Package>,
    repository: &Repo,
) -> anyhow::Result<Diff> {
    let package_path = package.crate_path();
    repository.checkout_head()?;
    let remote_crate = remote_crates.get(&package.name);
    let mut diff = Diff::new(remote_crate.is_some());
    if let Err(_err) = repository.checkout_last_commit_at_path(package_path) {
        // there are no commits for this package
        return Ok(diff);
    }
    loop {
        let current_commit_message = repository.current_commit_message()?;
        if let Some(remote_crate) = remote_crate {
            debug!("remote crate {} found", remote_crate.name);
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
    repository.checkout_head()?;
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
) -> anyhow::Result<Vec<(Package, Version)>> {
    let mut packages_to_update = vec![];
    for c in crates {
        let diff = get_diff(&c, remote_crates, repository)?;
        let current_version = &c.version;
        let next_version = c.version.next_from_diff(&diff);

        debug!("diff: {:?}, next_version: {}", &diff, next_version);
        if next_version != *current_version {
            packages_to_update.push((c, next_version));
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
fn get_remote_crates(
    remote_manifest: Option<&PathBuf>,
    local_crates: &[Package],
) -> anyhow::Result<BTreeMap<String, Package>> {
    let remote_crates = match remote_manifest {
        Some(manifest) => list_crates(manifest)?,
        None => {
            let local_crates_names: Vec<&str> =
                local_crates.iter().map(|c| c.name.as_str()).collect();
            crate::download::download_crates(&local_crates_names)?
        }
    };
    let remote_crates = remote_crates
        .into_iter()
        .map(|c| {
            let package_name = c.name.clone();
            (package_name, c)
        })
        .collect();
    Ok(remote_crates)
}

#[instrument]
fn update_versions(local_crates: &[(Package, Version)]) {
    for (package, next_version) in local_crates {
        let package_path = package.crate_path();
        set_version(package_path, next_version);
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
