mod git;
mod log;
mod version;

use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::Context;
use cargo_edit::VersionExt;
use cargo_metadata::Package;
use tracing::debug;

use crate::git::Repo;

#[derive(Debug)]
struct LocalPackage {
    package: Package,
    diff: Diff,
}

/// Difference between local and remote crate
#[derive(Debug)]
struct Diff {
    commits: Vec<String>,
    /// Whether the crate name exists in the remote crates or not
    remote_crate_exists: bool,
}

impl Diff {
    fn new(remote_crate_exists: bool) -> Self {
        Self {
            commits: vec![],
            remote_crate_exists,
        }
    }
}

enum SemVer {
    Major,
    Minor,
    Patch,
}

impl LocalPackage {
    /// Analyze commits and determine which part of version to increment based on
    /// [conventional commits](https://www.conventionalcommits.org/)
    fn version_increment(&self) -> Option<SemVer> {
        Some(SemVer::Minor)
    }
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
            manifest_path.pop();
            let crate_path: PathBuf = manifest_path.into_std_path_buf();
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

fn main() -> anyhow::Result<()> {
    install_dependencies()?;
    // TODO download in tmp directory
    //download_crate("rust-gh-example")?;
    let local_path = "/home/marco/me/proj/rust-gh-example2/Cargo.toml";
    let local_path = fs::canonicalize(local_path).context("local_path doesn't exist")?;
    let local_crates = list_crates(&local_path);
    let remote_crates = list_crates(&PathBuf::from(
        "/home/marco/me/proj/rust-gh-example/Cargo.toml",
    ));
    dbg!(&remote_crates);
    let mut local_crates = calculate_local_crates(local_crates.into_iter())?;
    let remote_crates = calculate_remote_crates(remote_crates.into_iter())?;
    dbg!(&local_crates);
    dbg!(&remote_crates);
    let repository = Repo::new(&local_path)?;

    for (package_path, package) in &mut local_crates {
        repository.checkout_head()?;
        loop {
            let current_commit_message = repository.current_commit_message()?;
            if let Some(remote_crate) = remote_crates.get(&package.package.name) {
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

    for (_, package) in &mut local_crates {
        let version_increment = package.version_increment();
        if let Some(increment) = version_increment {
            match increment {
                SemVer::Major => todo!(),
                SemVer::Minor => package.package.version.increment_minor(),
                SemVer::Patch => todo!(),
            }
        }
    }

    // pr command:
    // - go back commit by commit and for every local crate:
    //   - If the local crate was edited in that commit:
    //     - if the hash of that crate is the same of the remote crate, that local crate is done.
    //     - otherwise:
    //       - add the entry to the changelog of that crate.
    //       - bump the version of that crate according to the semantic versioning of the commit.
    // - raise PR

    // release command (probably this is already done in ):
    // - for every local_crate with a version != remote one:
    //   - publish crate
    //   - create a new tag with format `local_crate v*new_version*`
    // // Maybe the same or similar is done by :
    // // cargo workspaces publish  --from-git --token "${TOKEN}" --yes
    Ok(())
}

fn install_dependencies() -> anyhow::Result<()> {
    for program in ["cargo-clone", "sha1dir"] {
        Command::new("cargo").args(["install", program]).output()?;
    }
    Ok(())
}

fn list_crates(directory: &Path) -> Vec<Package> {
    cargo_edit::workspace_members(Some(directory)).unwrap()
}

fn download_crate(crate_name: &str) -> anyhow::Result<()> {
    Command::new("cargo").args(["clone", crate_name]).output()?;
    Ok(())
}

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
