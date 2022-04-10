use crate::{
    diff::Diff,
    registry_packages::{self, PackagesCollection},
    tmp_repo::TempRepo,
    version::NextVersionFromDiff,
    ChangelogBuilder, CARGO_TOML, CHANGELOG_FILENAME,
};
use anyhow::{anyhow, Context};
use cargo_metadata::{Package, Version};
use chrono::{Date, Utc};
use folder_compare::FolderCompare;
use fs_extra::dir;
use git_cmd::{self, Repo};
use std::{
    fs, io,
    path::{Path, PathBuf},
};
use tempfile::{tempdir, TempDir};
use tracing::{debug, info, instrument};

#[derive(Debug, Clone)]
pub struct UpdateRequest {
    /// The manifest of the project you want to update.
    local_manifest: PathBuf,
    /// Manifest of the project containing packages at the versions published in the Cargo registry.
    registry_manifest: Option<PathBuf>,
    /// Update just this package.
    single_package: Option<String>,
    /// If [`Option::Some`], changelog is updated.
    changelog_req: Option<ChangelogRequest>,
    /// Registry where the packages are stored.
    /// The registry name needs to be present in the Cargo config.
    /// If unspecified, crates.io is used.
    registry: Option<String>,
}

#[derive(Debug, Clone, Copy)]
pub struct ChangelogRequest {
    /// When the new release is published. If unspecified, current date is used.
    pub release_date: Option<Date<Utc>>,
}

fn canonical_local_manifest(local_manifest: &Path) -> io::Result<PathBuf> {
    let mut local_manifest = fs::canonicalize(local_manifest)?;
    if !local_manifest.ends_with(CARGO_TOML) {
        local_manifest.push(CARGO_TOML)
    }
    Ok(local_manifest)
}

impl UpdateRequest {
    pub fn new(local_manifest: impl AsRef<Path>) -> io::Result<Self> {
        Ok(Self {
            local_manifest: canonical_local_manifest(local_manifest.as_ref())?,
            registry_manifest: None,
            single_package: None,
            changelog_req: None,
            registry: None,
        })
    }

    pub fn set_local_manifest(self, local_manifest: impl AsRef<Path>) -> io::Result<Self> {
        Ok(Self {
            local_manifest: canonical_local_manifest(local_manifest.as_ref())?,
            ..self
        })
    }

    pub fn with_registry_project_manifest(self, registry_manifest: PathBuf) -> io::Result<Self> {
        let registry_manifest = fs::canonicalize(registry_manifest)?;
        Ok(Self {
            registry_manifest: Some(registry_manifest),
            ..self
        })
    }

    pub fn with_changelog(self, changelog_req: ChangelogRequest) -> Self {
        Self {
            changelog_req: Some(changelog_req),
            ..self
        }
    }

    pub fn with_registry(self, registry: String) -> Self {
        Self {
            registry: Some(registry),
            ..self
        }
    }

    pub fn with_single_package(self, package: String) -> Self {
        Self {
            single_package: Some(package),
            ..self
        }
    }

    pub fn local_manifest(&self) -> &Path {
        &self.local_manifest
    }

    pub fn registry_manifest(&self) -> Option<&Path> {
        self.registry_manifest.as_deref()
    }
}

/// Determine next version of packages
#[instrument]
pub fn next_versions(
    input: &UpdateRequest,
) -> anyhow::Result<(Vec<(Package, UpdateResult)>, TempRepo)> {
    let local_project = Project::new(input)?;
    let registry_packages = registry_packages::get_registry_packages(
        input.registry_manifest.as_ref(),
        &local_project.packages,
        input.registry.as_deref(),
    )?;

    let repository = local_project.get_repo()?;
    let packages_to_update = packages_to_update(
        local_project,
        &registry_packages,
        &repository.repo,
        input.changelog_req,
    )?;
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
        let manifest_dir = manifest_dir(manifest)?.to_path_buf();
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

pub struct UpdateResult {
    pub version: Version,
    pub changelog: Option<String>,
}

#[instrument(skip_all)]
fn packages_to_update(
    project: Project,
    registry_packages: &PackagesCollection,
    repository: &Repo,
    changelog_req: Option<ChangelogRequest>,
) -> anyhow::Result<Vec<(Package, UpdateResult)>> {
    repository.is_clean()?;
    debug!("calculating local packages");
    let mut packages_to_update = vec![];
    for p in project.packages {
        let diff = get_diff(&p, registry_packages, repository, &project.root)?;
        let current_version = &p.version;
        let next_version = p.version.next_from_diff(&diff);

        debug!("diff: {:?}, next_version: {}", &diff, next_version);
        if next_version != *current_version {
            info!("{}: next version is {next_version}", p.name);
            let changelog = changelog_req
                .map(|r| get_changelog(diff.commits.clone(), &next_version, r.release_date, &p))
                .transpose()?;

            let update_result = UpdateResult {
                version: next_version,
                changelog,
            };
            packages_to_update.push((p, update_result));
        }
    }
    Ok(packages_to_update)
}

fn get_changelog(
    commits: Vec<String>,
    next_version: &Version,
    release_date: Option<Date<Utc>>,
    package: &Package,
) -> anyhow::Result<String> {
    let mut changelog_builder = ChangelogBuilder::new(commits, next_version.to_string());
    if let Some(release_date) = release_date {
        changelog_builder = changelog_builder.with_release_date(release_date)
    }
    let new_changelog = changelog_builder.build();
    let changelog = match fs::read_to_string(package.changelog_path()?) {
        Ok(old_changelog) => new_changelog.prepend(old_changelog),
        Err(_err) => new_changelog.generate(), // Old changelog doesn't exist.
    };
    Ok(changelog)
}

#[instrument(
    skip_all,
    fields(package = %package.name)
)]
fn get_diff(
    package: &Package,
    registry_packages: &PackagesCollection,
    repository: &Repo,
    project_root: &Path,
) -> anyhow::Result<Diff> {
    let package_path = {
        let relative_path = package
            .package_path()?
            .strip_prefix(project_root)
            .context("error while retrieving package_path")?;
        repository.directory().join(relative_path)
    };
    repository.checkout_head()?;
    let registry_package = registry_packages.get_package(&package.name);
    let mut diff = Diff::new(registry_package.is_some());
    if let Err(_err) = repository.checkout_last_commit_at_path(&package_path) {
        info!("{}: there are no commits", package.name);
        return Ok(diff);
    }
    loop {
        let current_commit_message = repository.current_commit_message()?;
        if let Some(registry_package) = registry_package {
            debug!("package {} found in cargo registry", registry_package.name);
            let are_packages_equal = {
                let registry_package_path = registry_package.package_path()?;
                are_packages_equal(&package_path, registry_package_path)
            };
            if are_packages_equal {
                debug!(
                    "next version calculated starting from commit after `{current_commit_message}`"
                );
                if diff.commits.is_empty() {
                    info!("{}: already up to date", package.name);
                }
                // The local package is identical to the registry one, which means that
                // the package was published at this commit, so we will not count this commit
                // as part of the release.
                // We can process the next create.
                break;
            } else if registry_package.version != package.version {
                info!("{}: the local package has already a different version with respect to the registry package, so release-plz will not update it", package.name);
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
            debug!("there are no other commits");
            break;
        }
    }
    repository.checkout_head()?;
    Ok(diff)
}

pub fn are_packages_equal(local_package: &Path, registry_package: &Path) -> bool {
    let are_toml_same = || {
        // When a package is published to a cargo registry, the original `Cargo.toml` file is stored as
        // `Cargo.toml.orig`
        let cargo_orig = format!("{CARGO_TOML}.orig");
        are_file_equal(
            &local_package.join(CARGO_TOML),
            &registry_package.join(cargo_orig),
        )
        .unwrap_or(false)
    };
    let are_dir_same = || {
        let excluded = vec![
            ".git".to_string(),
            ".cargo_vcs_info.json".to_string(),
            CARGO_TOML.to_string(),
        ];
        let result = FolderCompare::new(local_package, registry_package, &excluded)
            .expect("cannot compare folders");
        result.changed_files.is_empty() && result.new_files.is_empty()
    };
    are_toml_same() && are_dir_same()
}

fn are_file_equal(first: &Path, second: &Path) -> io::Result<bool> {
    let first = fs::read_to_string(first)?;
    let second = fs::read_to_string(second)?;
    Ok(first == second)
}

pub fn public_packages(manifest: &Path) -> anyhow::Result<Vec<Package>> {
    let packages = cargo_edit::workspace_members(Some(manifest))
        .map_err(|e| anyhow!("cannot read workspace members in manifest {manifest:?}: {e}"))?
        .into_iter()
        // skip packages with `publish = false`
        .filter(|c| c.publish.is_none())
        .collect();
    Ok(packages)
}

pub trait PackagePath {
    fn package_path(&self) -> anyhow::Result<&Path>;
    fn changelog_path(&self) -> anyhow::Result<PathBuf> {
        let changelog_path = self.package_path()?.join(CHANGELOG_FILENAME);
        Ok(changelog_path)
    }
}

impl PackagePath for Package {
    fn package_path(&self) -> anyhow::Result<&Path> {
        manifest_dir(self.manifest_path.as_std_path())
    }
}

fn manifest_dir(manifest: &Path) -> anyhow::Result<&Path> {
    let manifest_dir = manifest.parent().ok_or_else(|| {
        anyhow!(
            "Cannot find directory where manifest {:?} is located",
            manifest
        )
    })?;
    Ok(manifest_dir)
}

pub fn copy_to_temp_dir(target: &Path) -> anyhow::Result<TempDir> {
    let tmp_dir = tempdir().context("cannot create temporary directory")?;
    dir::copy(target, tmp_dir.as_ref(), &dir::CopyOptions::default())
        .context(format!("cannot copy directory {target:?} to {tmp_dir:?}",))?;
    Ok(tmp_dir)
}
