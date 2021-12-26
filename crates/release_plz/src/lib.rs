mod git;
mod version;

use crate::{git::Repo, version::NextVersionFromDiff};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    process::Command,
};

use cargo_metadata::Package;
use tracing::debug;

#[derive(Debug)]
struct LocalPackage {
    package: Package,
    diff: Diff,
}

/// Difference between local and remote crate
#[derive(Debug)]
struct Diff {
    pub commits: Vec<String>,
    /// Whether the crate name exists in the remote crates or not
    pub remote_crate_exists: bool,
}

impl Diff {
    fn new(remote_crate_exists: bool) -> Self {
        Self {
            commits: vec![],
            remote_crate_exists,
        }
    }
}

/// Update a local rust project
pub fn update(local_manifest: &Path, remote_manifest: &Path) -> anyhow::Result<()> {
    let local_crates = list_crates(local_manifest);
    let remote_crates = list_crates(remote_manifest);
    let mut local_crates = calculate_local_crates(local_crates.into_iter())?;
    let remote_crates = calculate_remote_crates(remote_crates.into_iter())?;
    let mut local_path = local_manifest.to_path_buf();
    local_path.pop();
    let repository = Repo::new(&local_path)?;

    debug!("calculating local packages");
    for (package_path, package) in &mut local_crates {
        debug!("processing local package {}", package.package.name);
        repository.checkout_head()?;
        if let Err(_err) = repository.checkout_last_commit_at_path(package_path) {
            // there are no commits for this package
            break;
        }
        loop {
            let current_commit_message = repository.current_commit_message()?;
            if let Some(remote_crate) = remote_crates.get(&package.package.name) {
                debug!("remote crate {} found", remote_crate.package.name);
                package.diff.remote_crate_exists = true;
                let crate_hash = hash_dir(package_path)?;
                let same_hash = remote_crate.hash == crate_hash;
                if same_hash {
                    // The local crate is identical to the remote one, so go to the next create
                    break;
                } else {
                    package.diff.commits.push(current_commit_message.clone());
                }
            } else {
                package.diff.commits.push(current_commit_message.clone());
            }
            if let Err(_err) = repository.checkout_previous_commit_at_path(package_path) {
                // there are no other commits.
                break;
            }
        }
    }
    debug!("local packages calculated");

    for package in &mut local_crates.values() {
        let current_version = package.package.version.clone();
        let next_version = current_version.next_from_diff(&package.diff);
        if next_version != current_version {
            todo!("bump to {}", next_version);
        }
    }

    Ok(())
}

fn list_crates(directory: &Path) -> Vec<Package> {
    cargo_edit::workspace_members(Some(directory)).unwrap()
}

// TODO use dir_diff library
fn hash_dir(dir: impl AsRef<Path>) -> anyhow::Result<String> {
    let output = Command::new("sha1dir").arg(dir.as_ref()).output()?;
    let output = String::from_utf8(output.stdout)?;
    let sha1 = output
        .split(' ')
        .into_iter()
        .next()
        .expect("cannot calculate hash");

    Ok(sha1.to_string())
}

#[derive(Debug)]
struct RemotePackage {
    package: Package,
    hash: String,
}

fn calculate_local_crates(
    crates: impl Iterator<Item = Package>,
) -> anyhow::Result<BTreeMap<PathBuf, LocalPackage>> {
    crates
        .map(|c| {
            let mut manifest_path = c.manifest_path.clone();
            debug!("manifest path: {}", manifest_path);
            manifest_path.pop();
            let crate_path: PathBuf = manifest_path.into_std_path_buf();
            debug!("crate path: {:?}", crate_path);
            let local_package = LocalPackage {
                package: c,
                diff: Diff::new(false),
            };
            Ok((crate_path, local_package))
        })
        .collect()
}

/// Return BTreeMap with "package name" as key
fn calculate_remote_crates(
    crates: impl Iterator<Item = Package>,
) -> anyhow::Result<BTreeMap<String, RemotePackage>> {
    crates
        .map(|c| {
            let mut manifest_path = c.manifest_path.clone();
            manifest_path.pop();
            let crate_path: PathBuf = manifest_path.into_std_path_buf();
            let hash = hash_dir(&crate_path)?;
            let remote_package = RemotePackage { package: c, hash };
            let package_name = remote_package.package.name.clone();
            Ok((package_name, remote_package))
        })
        .collect()
}
