use crate::{diff::Diff, download, tmp_repo::TempRepo, version::NextVersionFromDiff, CARGO_TOML};
use anyhow::{anyhow, Context};
use cargo_metadata::{Package, Version};
use folder_compare::FolderCompare;
use fs_extra::dir;
use git_cmd::{self, Repo};
use std::{
    collections::BTreeMap,
    fs, io,
    path::{Path, PathBuf},
};
use tempfile::{tempdir, TempDir};
use tracing::{debug, info, instrument};

#[derive(Debug)]
pub struct UpdateRequest {
    local_manifest: PathBuf,
    remote_manifest: Option<PathBuf>,
    /// Update just this package.
    single_package: Option<String>,
}

impl UpdateRequest {
    pub fn new(local_manifest: PathBuf) -> io::Result<Self> {
        let mut local_manifest = fs::canonicalize(local_manifest)?;
        if !local_manifest.ends_with(CARGO_TOML) {
            local_manifest.push(CARGO_TOML)
        }
        Ok(Self {
            local_manifest,
            remote_manifest: None,
            single_package: None,
        })
    }

    pub fn with_remote_manifest(self, remote_manifest: PathBuf) -> io::Result<Self> {
        let remote_manifest = fs::canonicalize(remote_manifest)?;
        Ok(Self {
            remote_manifest: Some(remote_manifest),
            ..self
        })
    }

    pub fn with_single_package(self, package: String) -> io::Result<Self> {
        Ok(Self {
            single_package: Some(package),
            ..self
        })
    }

    pub fn local_manifest(&self) -> &Path {
        &self.local_manifest
    }

    pub fn remote_manifest(&self) -> Option<&Path> {
        self.remote_manifest.as_deref()
    }
}

/// Determine next version of packages
#[instrument]
pub fn next_versions(input: &UpdateRequest) -> anyhow::Result<(Vec<(Package, Version)>, TempRepo)> {
    let local_project = Project::new(input)?;
    let remote_packages =
        get_remote_packages(input.remote_manifest.as_ref(), &local_project.packages)?;

    let repository = local_project.get_repo()?;

    let packages_to_update = packages_to_update(local_project, &remote_packages, &repository.repo)?;
    debug!("packages to update: {:?}", &packages_to_update);
    Ok((packages_to_update, repository))
}

struct Project {
    packages: Vec<Package>,
    /// Project root directory
    root: PathBuf,
    /// Directory containing the project manifest
    manifest_dir: PathBuf,
}

impl Project {
    fn new(input: &UpdateRequest) -> anyhow::Result<Self> {
        let manifest = &input.local_manifest;
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
        let mut packages = public_packages(manifest)?;
        if let Some(pac) = &input.single_package {
            packages = packages.into_iter().filter(|p| &p.name == pac).collect();
        }

        anyhow::ensure!(!packages.is_empty(), "no public packages found");

        Ok(Self {
            packages,
            root,
            manifest_dir,
        })
    }

    /// Copy this project in a temporary repository return the repository.
    /// We copy the project in another directory in order to avoid altering it.
    fn get_repo(&self) -> anyhow::Result<TempRepo> {
        let tmp_project_root = copy_to_temp_dir(&self.root)?;
        let tmp_manifest_dir = {
            let parent_root = self.root.parent().context("cannot determine parent root")?;
            let relative_manifest_dir = self
                .manifest_dir
                .strip_prefix(parent_root)
                .context("cannot strip prefix for manifest dir")?;
            debug!("relative_manifest_dir: {relative_manifest_dir:?}");
            tmp_project_root.as_ref().join(relative_manifest_dir)
        };
        debug!("tmp_manifest_dir: {tmp_manifest_dir:?}");

        let repository = TempRepo::new(tmp_project_root, &tmp_manifest_dir)?;
        Ok(repository)
    }
}

/// Return [`BTreeMap`] with "package name" as key
fn get_remote_packages(
    remote_manifest: Option<&PathBuf>,
    local_packages: &[Package],
) -> anyhow::Result<BTreeMap<String, Package>> {
    let remote_packages = match remote_manifest {
        Some(manifest) => public_packages(manifest)?,
        None => {
            let local_packages_names: Vec<&str> =
                local_packages.iter().map(|c| c.name.as_str()).collect();
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

#[instrument(skip_all)]
fn packages_to_update(
    project: Project,
    remote_packages: &BTreeMap<String, Package>,
    repository: &Repo,
) -> anyhow::Result<Vec<(Package, Version)>> {
    debug!("calculating local packages");
    let mut packages_to_update = vec![];
    for p in project.packages {
        let diff = get_diff(&p, remote_packages, repository, &project.root)?;
        let current_version = &p.version;
        let next_version = p.version.next_from_diff(&diff);

        debug!("diff: {:?}, next_version: {}", &diff, next_version);
        if next_version != *current_version {
            packages_to_update.push((p, next_version));
        }
    }
    Ok(packages_to_update)
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
        info!("there are no commits for this package");
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
                info!(
                    "next version calculated starting from commit after `{current_commit_message}`"
                );
                // The local package is identical to the remote one, which means that
                // the package was published at this commit, so we will not count this commit
                // as part of the release.
                // We can process the next create.
                break;
            } else if remote_package.version != package.version {
                info!("the local package has already a different version with respect to the remote package, so release-plz will not update it");
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
            info!("there are no other commits");
            break;
        }
    }
    repository.checkout_head()?;
    Ok(diff)
}

pub fn are_packages_equal(first: &Path, second: &Path) -> bool {
    let excluded = vec![".git".to_string(), ".cargo_vcs_info.json".to_string()];
    let result = FolderCompare::new(first, second, &excluded).unwrap();
    result.changed_files.is_empty() && result.new_files.is_empty()
}

fn public_packages(directory: &Path) -> anyhow::Result<Vec<Package>> {
    let packages = cargo_edit::workspace_members(Some(directory))
        .map_err(|e| anyhow!("cannot read workspace members: {e}"))?
        .into_iter()
        // skip packages with `publish = false`
        .filter(|c| c.publish.is_none())
        .collect();
    Ok(packages)
}

pub trait PackagePath {
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

pub fn copy_to_temp_dir(target: &Path) -> anyhow::Result<TempDir> {
    let tmp_dir = tempdir().context("cannot create temporary directory")?;
    dir::copy(target, tmp_dir.as_ref(), &dir::CopyOptions::default())
        .context(format!("cannot copy directory {target:?} to {tmp_dir:?}",))?;
    Ok(tmp_dir)
}
