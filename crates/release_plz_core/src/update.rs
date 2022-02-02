use crate::{diff::Diff, download, version::NextVersionFromDiff};
use anyhow::{anyhow, Context};
use cargo_edit::LocalManifest;
use cargo_metadata::{Package, Version};
use folder_compare::FolderCompare;
use fs_extra::dir;
use git_cmd::{self, Repo};
use std::{
    collections::BTreeMap,
    fs, io,
    path::{Path, PathBuf},
};
use tempfile::tempdir;
use tracing::{debug, instrument};

#[derive(Debug)]
pub struct UpdateRequest {
    local_manifest: PathBuf,
    remote_manifest: Option<PathBuf>,
}

impl UpdateRequest {
    pub fn new(local_manifest: PathBuf) -> io::Result<Self> {
        let local_manifest = fs::canonicalize(local_manifest)?;
        Ok(Self {
            local_manifest,
            remote_manifest: None,
        })
    }

    pub fn with_remote_manifest(self, remote_manifest: PathBuf) -> io::Result<Self> {
        let remote_manifest = fs::canonicalize(remote_manifest)?;
        Ok(Self {
            remote_manifest: Some(remote_manifest),
            ..self
        })
    }
}

struct Project {
    packages: Vec<Package>,
    /// Project root directory
    root: PathBuf,
    /// Directory containing the project manifest
    manifest_dir: PathBuf,
}

impl Project {
    fn new(manifest: &Path) -> anyhow::Result<Self> {
        let manifest_dir = manifest
            .parent()
            .ok_or_else(|| {
                anyhow!(
                    "cannot find directory where manifest {:?} is located",
                    manifest
                )
            })?
            .to_path_buf();
        debug!("manifest_dir: {manifest_dir:?}");
        let root = {
            let project_root =
                git_cmd::git_in_dir(&manifest_dir, &["rev-parse", "--show-toplevel"])?;
            PathBuf::from(project_root)
        };
        debug!("project_root: {root:?}");
        Ok(Self {
            packages: list_packages(manifest)?,
            root,
            manifest_dir,
        })
    }

    /// Copy this project in a temporary repository located in `tmp_project_root` and return the repository.
    fn get_repo(&self, tmp_project_root: &Path) -> anyhow::Result<Repo> {
        dir::copy(&self.root, tmp_project_root, &dir::CopyOptions::default()).context(format!(
            "cannot copy directory {:?} to {:?}",
            self.root, tmp_project_root
        ))?;

        let tmp_manifest_dir = {
            let parent_root = self.root.parent().context("cannot determine parent root")?;
            let relative_manifest_dir = self
                .manifest_dir
                .strip_prefix(parent_root)
                .context("cannot strip prefix for manifest dir")?;
            debug!("relative_manifest_dir: {relative_manifest_dir:?}");
            tmp_project_root.join(relative_manifest_dir)
        };
        debug!("tmp_manifest_dir: {tmp_manifest_dir:?}");

        let repository = Repo::new(&tmp_manifest_dir)?;
        Ok(repository)
    }
}

/// Determine next version of packages
#[instrument]
pub fn next_versions(input: &UpdateRequest) -> anyhow::Result<(Vec<(Package, Version)>, Repo)> {
    let local_project = Project::new(&input.local_manifest)?;
    let remote_packages =
        get_remote_packages(input.remote_manifest.as_ref(), &local_project.packages)?;

    // copy the repository into a temporary directory, so that we are not sure we don't alter the original one
    let tmp_project_root = tempdir().context("cannot create temporary directory")?;
    let repository = local_project.get_repo(tmp_project_root.as_ref())?;

    let packages_to_update = packages_to_update(local_project, &remote_packages, &repository)?;
    debug!("packages to update: {:?}", &packages_to_update);
    Ok((packages_to_update, repository))
}

/// Update a local rust project
#[instrument]
pub fn update(input: &UpdateRequest) -> anyhow::Result<(Vec<(Package, Version)>, Repo)> {
    let (packages_to_update, repository) = next_versions(input)?;
    update_versions(&packages_to_update);
    Ok((packages_to_update, repository))
}

#[instrument(
    skip_all,
    fields(package = %package.name)
)]
fn get_diff(
    package: &Package,
    remote_packages: &BTreeMap<String, Package>,
    repository: &Repo,
    project_root: &Path,
) -> anyhow::Result<Diff> {
    let package_path = {
        let relative_path = package
            .package_path()
            .strip_prefix(project_root)
            .context("error while retrieving package_path")?;
        repository.directory().join(relative_path)
    };
    repository.checkout_head()?;
    let remote_package = remote_packages.get(&package.name);
    let mut diff = Diff::new(remote_package.is_some());
    if let Err(_err) = repository.checkout_last_commit_at_path(&package_path) {
        // there are no commits for this package
        return Ok(diff);
    }
    loop {
        let current_commit_message = repository.current_commit_message()?;
        if let Some(remote_package) = remote_package {
            debug!("remote package {} found", remote_package.name);
            let are_packages_equal = {
                let remote_path = remote_package
                    .manifest_path
                    .parent()
                    .context("cannot find parent directory")?;
                are_packages_equal(&package_path, remote_path.as_ref())
            };
            if are_packages_equal {
                debug!("packages are equal");
                // The local package is identical to the remote one, which means that
                // the package was published at this commit, so we will not count this commit
                // as part of the release.
                // We can process the next create.
                break;
            } else if remote_package.version != package.version {
                debug!("the local package {} has already a different version with respect to the remote package, so release-plz will not update it", package.name);
                break;
            } else {
                debug!("packages are different");
                // At this point of the git history, the two packages are different,
                // which means that this commit is not present in the published package.
                diff.commits.push(current_commit_message.clone());
            }
        } else {
            diff.commits.push(current_commit_message.clone());
        }
        if let Err(_err) = repository.checkout_previous_commit_at_path(&package_path) {
            // there are no other commits.
            break;
        }
    }
    repository.checkout_head()?;
    Ok(diff)
}

pub fn are_packages_equal(first: &Path, second: &Path) -> bool {
    let excluded = vec![".git".to_string()];
    let result = FolderCompare::new(first, second, &excluded).unwrap();
    result.changed_files.is_empty() && result.new_files.is_empty()
}

#[instrument(skip_all)]
fn packages_to_update(
    project: Project,
    remote_packages: &BTreeMap<String, Package>,
    repository: &Repo,
) -> anyhow::Result<Vec<(Package, Version)>> {
    debug!("calculating local packages");
    let mut packages_to_update = vec![];
    for c in project.packages {
        let diff = get_diff(&c, remote_packages, repository, &project.root)?;
        let current_version = &c.version;
        let next_version = c.version.next_from_diff(&diff);

        debug!("diff: {:?}, next_version: {}", &diff, next_version);
        if next_version != *current_version {
            packages_to_update.push((c, next_version));
        }
    }
    Ok(packages_to_update)
}

trait PackagePath {
    fn package_path(&self) -> &Path;
}

impl PackagePath for Package {
    fn package_path(&self) -> &Path {
        self.manifest_path
            .parent()
            .expect("Cannot find directory containing Cargo.toml file")
            .as_std_path()
    }
}

/// Return [`BTreeMap`] with "package name" as key
fn get_remote_packages(
    remote_manifest: Option<&PathBuf>,
    local_packages: &[Package],
) -> anyhow::Result<BTreeMap<String, Package>> {
    let remote_packages = match remote_manifest {
        Some(manifest) => list_packages(manifest)?,
        None => {
            let local_packages_names: Vec<&str> = local_packages
                .iter()
                // skip packages with `publish = false`
                .filter(|c| c.publish.is_none())
                .map(|c| c.name.as_str())
                .collect();
            download::download_packages(&local_packages_names)?
        }
    };
    let remote_packages = remote_packages
        .into_iter()
        .map(|c| {
            let package_name = c.name.clone();
            (package_name, c)
        })
        .collect();
    Ok(remote_packages)
}

#[instrument]
fn update_versions(local_packages: &[(Package, Version)]) {
    for (package, next_version) in local_packages {
        let package_path = package.package_path();
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

fn list_packages(directory: &Path) -> anyhow::Result<Vec<Package>> {
    cargo_edit::workspace_members(Some(directory))
        .map_err(|e| anyhow!("cannot read workspace members: {e}"))
}
