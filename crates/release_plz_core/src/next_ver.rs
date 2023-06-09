use crate::{
    changelog_parser::{self, ChangelogRelease},
    diff::Diff,
    lock_compare,
    package_compare::are_packages_equal,
    package_path::{manifest_dir, PackagePath},
    registry_packages::{self, PackagesCollection},
    repo_url::RepoUrl,
    semver_check::{self, SemverCheck},
    tmp_repo::TempRepo,
    version::NextVersionFromDiff,
    ChangelogBuilder, PackagesUpdate, CARGO_TOML, CHANGELOG_FILENAME,
};
use anyhow::{anyhow, Context};
use cargo_metadata::{semver::Version, Dependency, Package};
use cargo_utils::{upgrade_requirement, LocalManifest};
use chrono::NaiveDate;
use fs_extra::dir;
use git_cliff_core::config::Config as GitCliffConfig;
use git_cmd::{self, Repo};
use next_version::NextVersion;
use rayon::prelude::{IntoParallelRefMutIterator, ParallelIterator};
use regex::Regex;
use std::{
    collections::BTreeMap,
    fs, io,
    path::{Path, PathBuf},
};
use tempfile::{tempdir, TempDir};
use tracing::{debug, info, instrument, warn};

#[derive(Debug, Clone)]
pub struct UpdateRequest {
    /// The manifest of the project you want to update.
    local_manifest: PathBuf,
    /// Manifest of the project containing packages at the versions published in the Cargo registry.
    registry_manifest: Option<PathBuf>,
    /// Update just this package.
    single_package: Option<String>,
    /// Changelog options.
    changelog_req: ChangelogRequest,
    /// Registry where the packages are stored.
    /// The registry name needs to be present in the Cargo config.
    /// If unspecified, crates.io is used.
    registry: Option<String>,
    /// - If true, update all the dependencies in Cargo.lock by running `cargo update`.
    /// - If false, updates the workspace packages in Cargo.lock by running `cargo update --workspace`.
    dependencies_update: bool,
    /// Allow dirty working directories to be updated.
    /// The uncommitted changes will be part of the update.
    allow_dirty: bool,
    /// Repository Url. If present, the new changelog entry contains a link to the diff between the old and new version.
    /// Format: `https://{repo_host}/{repo_owner}/{repo_name}/compare/{old_tag}...{new_tag}`.
    repo_url: Option<RepoUrl>,
    /// Package-specific configurations.
    packages_config: PackagesConfig,
}

#[derive(Debug, Clone, Default)]
struct PackagesConfig {
    /// Config for packages that don't have a specific configuration.
    default: UpdateConfig,
    /// Configurations that override `default`.
    /// The key is the package name.
    overrides: BTreeMap<String, PackageUpdateConfig>,
}

impl From<UpdateConfig> for PackageUpdateConfig {
    fn from(config: UpdateConfig) -> Self {
        Self {
            generic: config,
            changelog_path: None,
        }
    }
}

impl PackagesConfig {
    fn get(&self, package_name: &str) -> PackageUpdateConfig {
        self.overrides
            .get(package_name)
            .cloned()
            .unwrap_or(self.default.clone().into())
    }

    fn set_default(&mut self, config: UpdateConfig) {
        self.default = config;
    }

    fn set(&mut self, package_name: String, config: PackageUpdateConfig) {
        self.overrides.insert(package_name, config);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateConfig {
    /// Controls when to run cargo-semver-checks.
    /// Note: You can only run cargo-semver-checks if the package contains a library.
    ///       For example, if it has a `lib.rs` file.
    pub semver_check: bool,
    /// Whether to create/update changelog or not.
    /// Default: `true`.
    pub changelog_update: bool,
}

/// Package-specific config
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PackageUpdateConfig {
    /// config that can be applied by default to all packages.
    pub generic: UpdateConfig,
    /// The changelog path can only be specified for a single package.
    /// I.e. it cannot be applied to `[workspace]` configuration.
    /// This path needs to be a relative path to the Cargo.toml of the project.
    /// I.e. if you have a workspace, it needs to be relative to the workspace root.
    pub changelog_path: Option<PathBuf>,
}

impl PackageUpdateConfig {
    pub fn semver_check(&self) -> bool {
        self.generic.semver_check
    }

    pub fn should_update_changelog(&self) -> bool {
        self.generic.changelog_update
    }
}

impl Default for UpdateConfig {
    fn default() -> Self {
        Self {
            semver_check: true,
            changelog_update: true,
        }
    }
}

impl UpdateConfig {
    pub fn with_semver_check(self, semver_check: bool) -> Self {
        Self {
            semver_check,
            ..self
        }
    }

    pub fn with_changelog_update(self, changelog_update: bool) -> Self {
        Self {
            changelog_update,
            ..self
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct ChangelogRequest {
    /// When the new release is published. If unspecified, current date is used.
    pub release_date: Option<NaiveDate>,
    pub changelog_config: Option<GitCliffConfig>,
}

fn canonical_local_manifest(local_manifest: &Path) -> io::Result<PathBuf> {
    let mut local_manifest = dunce::canonicalize(local_manifest)?;
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
            changelog_req: ChangelogRequest::default(),
            registry: None,
            dependencies_update: false,
            allow_dirty: false,
            repo_url: None,
            packages_config: PackagesConfig::default(),
        })
    }

    pub fn changelog_path(&self, package: &Package) -> PathBuf {
        let config = self.get_package_config(&package.name);
        config
            .changelog_path
            .map(|p| self.local_manifest.parent().unwrap().join(p))
            .unwrap_or_else(|| {
                package
                    .package_path()
                    .expect("can't determine package path")
                    .join(CHANGELOG_FILENAME)
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

    pub fn with_changelog_req(self, changelog_req: ChangelogRequest) -> Self {
        Self {
            changelog_req,
            ..self
        }
    }

    /// Set update config for all packages.
    pub fn with_default_package_config(mut self, config: UpdateConfig) -> Self {
        self.packages_config.set_default(config);
        self
    }

    /// Set update config for a specific package.
    pub fn with_package_config(
        mut self,
        package: impl Into<String>,
        config: PackageUpdateConfig,
    ) -> Self {
        self.packages_config.set(package.into(), config);
        self
    }

    pub fn get_package_config(&self, package: &str) -> PackageUpdateConfig {
        self.packages_config.get(package)
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

    pub fn with_repo_url(self, repo_url: RepoUrl) -> Self {
        Self {
            repo_url: Some(repo_url),
            ..self
        }
    }

    pub fn local_manifest_dir(&self) -> anyhow::Result<&Path> {
        self.local_manifest
            .parent()
            .context("wrong local manifest path")
    }

    pub fn local_manifest(&self) -> &Path {
        &self.local_manifest
    }

    pub fn registry_manifest(&self) -> Option<&Path> {
        self.registry_manifest.as_deref()
    }

    pub fn with_dependencies_update(self, dependencies_update: bool) -> Self {
        Self {
            dependencies_update,
            ..self
        }
    }

    pub fn should_update_dependencies(&self) -> bool {
        self.dependencies_update
    }

    pub fn with_allow_dirty(self, allow_dirty: bool) -> Self {
        Self {
            allow_dirty,
            ..self
        }
    }

    pub fn repo_url(&self) -> Option<&RepoUrl> {
        self.repo_url.as_ref()
    }
}

/// Determine next version of packages
#[instrument]
pub fn next_versions(input: &UpdateRequest) -> anyhow::Result<(PackagesUpdate, TempRepo)> {
    let local_project = Project::new(&input.local_manifest, input.single_package.as_deref())?;
    let updater = Updater {
        project: &local_project,
        req: input,
    };
    let registry_packages = registry_packages::get_registry_packages(
        input.registry_manifest.as_ref(),
        &local_project.publishable_packages(),
        input.registry.as_deref(),
    )?;

    let repository = local_project.get_repo()?;
    if !input.allow_dirty {
        repository.repo.is_clean()?;
    }
    let packages_to_update = updater.packages_to_update(
        &registry_packages,
        &repository.repo,
        &local_project.workspace_packages(),
    )?;
    Ok((packages_to_update, repository))
}

#[derive(Debug)]
pub struct Project {
    /// Publishable packages.
    packages: Vec<Package>,
    /// Project root directory
    root: PathBuf,
    /// Directory containing the project manifest
    manifest_dir: PathBuf,
    /// The project contains more than one public package.
    /// Not affected by `single_package` option.
    contains_multiple_pub_packages: bool,
}

impl Project {
    pub fn new(local_manifest: &Path, single_package: Option<&str>) -> anyhow::Result<Self> {
        let manifest = &local_manifest;
        let manifest_dir = manifest_dir(manifest)?.to_path_buf();
        debug!("manifest_dir: {manifest_dir:?}");
        let root = {
            let project_root =
                git_cmd::git_in_dir(&manifest_dir, &["rev-parse", "--show-toplevel"])?;
            PathBuf::from(project_root)
        };
        debug!("project_root: {root:?}");
        let mut packages = workspace_packages(manifest)?;
        let contains_multiple_pub_packages = packages.len() > 1;
        if let Some(pac) = single_package {
            packages.retain(|p| p.name == pac);
        }

        anyhow::ensure!(!packages.is_empty(), "no public packages found");

        Ok(Self {
            packages,
            root,
            manifest_dir,
            contains_multiple_pub_packages,
        })
    }

    pub fn publishable_packages(&self) -> Vec<&Package> {
        self.packages
            .iter()
            .filter(|p| p.is_publishable())
            .collect()
    }

    /// Get all packages, including non-publishable.
    pub fn workspace_packages(&self) -> Vec<&Package> {
        self.packages.iter().collect()
    }

    /// Copy this project in a temporary repository and return the repository.
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

    pub fn git_tag(&self, package_name: &str, version: &str) -> String {
        if self.contains_multiple_pub_packages {
            format!("{package_name}-v{version}")
        } else {
            format!("v{version}")
        }
    }

    pub fn cargo_lock_path(&self) -> PathBuf {
        self.root.join("Cargo.lock")
    }
}

#[derive(Debug, Clone)]
pub struct UpdateResult {
    pub version: Version,
    pub changelog: Option<String>,
    pub semver_check: SemverCheck,
}

impl UpdateResult {
    pub fn last_changes(&self) -> anyhow::Result<Option<ChangelogRelease>> {
        match &self.changelog {
            Some(c) => changelog_parser::last_release_from_str(c),
            None => Ok(None),
        }
    }
}

pub struct Updater<'a> {
    pub project: &'a Project,
    pub req: &'a UpdateRequest,
}

impl Updater<'_> {
    #[instrument(skip_all)]
    fn packages_to_update(
        &self,
        registry_packages: &PackagesCollection,
        repository: &Repo,
        workspace_packages: &[&Package],
    ) -> anyhow::Result<PackagesUpdate> {
        debug!("calculating local packages");

        let packages_diffs =
            self.get_packages_diffs(registry_packages, repository, workspace_packages)?;
        let mut packages_to_check_for_deps: Vec<&Package> = vec![];
        let mut packages_to_update = PackagesUpdate { updates: vec![] };
        for (p, diff) in packages_diffs {
            let current_version = p.version.clone();
            let next_version = p.version.next_from_diff(&diff);

            debug!("diff: {:?}, next_version: {}", &diff, next_version);
            if next_version != current_version || !diff.registry_package_exists {
                info!(
                    "{}: next version is {next_version}{}",
                    p.name,
                    diff.semver_check.outcome_str()
                );
                let update_result =
                    self.update_result(diff.commits, next_version, p, diff.semver_check)?;

                packages_to_update.updates.push((p.clone(), update_result));
            } else if diff.is_version_published {
                packages_to_check_for_deps.push(p);
            }
        }

        let changed_packages: Vec<(&Package, &Version)> = packages_to_update
            .updates
            .iter()
            .map(|(p, u)| (p, &u.version))
            .collect();
        let dependent_packages =
            self.dependent_packages(&packages_to_check_for_deps, &changed_packages)?;
        packages_to_update.updates.extend(dependent_packages);
        Ok(packages_to_update)
    }

    fn get_packages_diffs(
        &self,
        registry_packages: &PackagesCollection,
        repository: &Repo,
        workspace_packages: &[&Package],
    ) -> anyhow::Result<Vec<(&Package, Diff)>> {
        // Store diff for each package. This operation is not thread safe, so we do it in one
        // package at a time.
        let packages_diffs_res: anyhow::Result<Vec<(&Package, Diff)>> = self
            .project
            .publishable_packages()
            .iter()
            .map(|&p| {
                let diff = get_diff(
                    p,
                    registry_packages,
                    repository,
                    self.project,
                    workspace_packages,
                )?;
                Ok((p, diff))
            })
            .collect();

        let mut packages_diffs = packages_diffs_res?;
        let semver_check_result: anyhow::Result<()> =
            packages_diffs.par_iter_mut().try_for_each(|(p, diff)| {
                let registry_package = registry_packages.get_package(&p.name);
                if let Some(registry_package) = registry_package {
                    let package_path = get_package_path(p, repository, &self.project.root)
                        .context("can't retrieve package path")?;
                    let run_semver_check = self.req.get_package_config(&p.name).semver_check();
                    if should_check_semver(p, run_semver_check) && diff.should_update_version() {
                        let registry_package_path = registry_package
                            .package_path()
                            .context("can't retrieve registry package path")?;
                        let semver_check =
                            semver_check::run_semver_check(&package_path, registry_package_path)
                                .context("error while running cargo-semver-checks")?;
                        diff.set_semver_check(semver_check);
                    }
                }
                Ok(())
            });
        semver_check_result?;

        Ok(packages_diffs)
    }

    /// Return the packages that depend on the `changed_packages`.
    fn dependent_packages(
        &self,
        packages_to_check_for_deps: &[&Package],
        changed_packages: &[(&Package, &Version)],
    ) -> anyhow::Result<Vec<(Package, UpdateResult)>> {
        let packages_to_update = packages_to_check_for_deps
            .iter()
            .filter_map(|p| match p.dependencies_to_update(changed_packages) {
                Ok(deps) => {
                    if deps.is_empty() {
                        None
                    } else {
                        Some((p, deps))
                    }
                }
                Err(_e) => None,
            })
            .map(|(&p, deps)| {
                let deps: Vec<&str> = deps.iter().map(|d| d.name.as_str()).collect();
                let change = format!(
                    "chore: updated the following local packages: {}",
                    deps.join(", ")
                );
                let next_version = { p.version.increment_patch() };
                info!(
                    "{}: dependencies changed. Next version is {next_version}",
                    p.name
                );
                Ok((
                    p.clone(),
                    self.update_result(vec![change], next_version, p, SemverCheck::Skipped)?,
                ))
            })
            .collect::<anyhow::Result<Vec<_>>>()?;
        Ok(packages_to_update)
    }

    fn update_result(
        &self,
        commits: Vec<String>,
        version: Version,
        package: &Package,
        semver_check: SemverCheck,
    ) -> anyhow::Result<UpdateResult> {
        let release_link = {
            let prev_tag = self
                .project
                .git_tag(&package.name, &package.version.to_string());
            let next_tag = self.project.git_tag(&package.name, &version.to_string());
            self.req
                .repo_url
                .as_ref()
                .map(|r| r.git_release_link(&prev_tag, &next_tag))
        };

        let pr_link = self.req.repo_url.as_ref().map(|r| r.git_pr_link());

        lazy_static::lazy_static! {
            // match PR/issue numbers, e.g. `#123`
            static ref PR_RE: Regex = Regex::new("#(\\d+)").unwrap();
        }
        let changelog = {
            let cfg = self.req.get_package_config(package.name.as_str());
            let changelog_req = cfg
                .should_update_changelog()
                .then_some(self.req.changelog_req.clone());
            let old_changelog = fs::read_to_string(self.req.changelog_path(package)).ok();
            let commits_titles: Vec<String> = commits
                .iter()
                // only take commit title
                .filter_map(|c| c.lines().next())
                // replace #123 with [#123](https://link_to_pr).
                // If the number refers to an issue, GitHub redirects the PR link to the issue link.
                .map(|c| {
                    if let Some(pr_link) = &pr_link {
                        let result = PR_RE.replace_all(c, format!("[#$1]({pr_link}/$1)"));
                        result.to_string()
                    } else {
                        c.to_string()
                    }
                })
                .collect();
            changelog_req
                .map(|r| {
                    get_changelog(
                        commits_titles,
                        &version,
                        Some(r),
                        old_changelog,
                        release_link,
                    )
                })
                .transpose()
        }?;

        Ok(UpdateResult {
            version,
            changelog,
            semver_check,
        })
    }
}

fn get_changelog(
    commits: Vec<String>,
    next_version: &Version,
    changelog_req: Option<ChangelogRequest>,
    old_changelog: Option<String>,
    release_link: Option<String>,
) -> anyhow::Result<String> {
    let mut changelog_builder = ChangelogBuilder::new(commits, next_version.to_string());
    if let Some(changelog_req) = changelog_req {
        if let Some(release_date) = changelog_req.release_date {
            changelog_builder = changelog_builder.with_release_date(release_date)
        }
        if let Some(config) = changelog_req.changelog_config {
            changelog_builder = changelog_builder.with_config(config)
        }
        if let Some(link) = release_link {
            changelog_builder = changelog_builder.with_release_link(link)
        }
    }
    let new_changelog = changelog_builder.build();
    let changelog = match old_changelog {
        Some(old_changelog) => new_changelog.prepend(old_changelog)?,
        None => new_changelog.generate(), // Old changelog doesn't exist.
    };
    Ok(changelog)
}

fn get_package_path(
    package: &Package,
    repository: &Repo,
    project_root: &Path,
) -> anyhow::Result<PathBuf> {
    let package_path = package.package_path()?;
    get_repo_path(package_path, repository, project_root)
}

fn get_repo_path(
    old_path: &Path,
    repository: &Repo,
    project_root: &Path,
) -> anyhow::Result<PathBuf> {
    let relative_path = old_path
        .strip_prefix(project_root)
        .context("error while retrieving package_path: project root not found")?;
    let result_path = repository.directory().join(relative_path);

    Ok(result_path)
}

/// This operation is not thread-safe, because we do `git checkout` on the repository.
#[instrument(
    skip_all,
    fields(package = %package.name)
)]
fn get_diff(
    package: &Package,
    registry_packages: &PackagesCollection,
    repository: &Repo,
    project: &Project,
    workspace_packages: &[&Package],
) -> anyhow::Result<Diff> {
    let package_path = get_package_path(package, repository, &project.root)?;

    repository.checkout_head()?;
    let registry_package = registry_packages.get_package(&package.name);
    let mut diff = Diff::new(registry_package.is_some());
    if let Err(err) = repository.checkout_last_commit_at_path(&package_path) {
        if err
            .to_string()
            .contains("Your local changes to the following files would be overwritten")
        {
            return Err(err.context("The allow-dirty option can't be used in this case"));
        } else {
            info!("{}: there are no commits", package.name);
            return Ok(diff);
        }
    }
    let ignored_dirs: anyhow::Result<Vec<PathBuf>> = workspace_packages
        .iter()
        .filter(|p| p.name != package.name)
        .map(|p| get_package_path(p, repository, &project.root))
        .collect();
    let ignored_dirs = ignored_dirs?;

    let tag_commit = {
        let git_tag = project.git_tag(&package.name, &package.version.to_string());
        repository.get_tag_commit(&git_tag)
    };
    loop {
        let current_commit_message = repository.current_commit_message()?;
        let current_commit_hash = repository.current_commit_hash()?;
        if let Some(registry_package) = registry_package {
            debug!("package {} found in cargo registry", registry_package.name);
            let registry_package_path = registry_package.package_path()?;
            let are_packages_equal =
                are_packages_equal(&package_path, registry_package_path, ignored_dirs.clone())
                    .context("cannot compare packages")?;
            if are_packages_equal
                || is_commit_too_old(repository, tag_commit.as_deref(), &current_commit_hash)
            {
                debug!(
                    "next version calculated starting from commits after `{current_commit_hash}`"
                );
                if diff.commits.is_empty() {
                    let are_dependencies_updated = are_toml_dependencies_updated(
                        &registry_package.dependencies,
                        &package.dependencies,
                    )
                        || lock_compare::are_lock_dependencies_updated(
                            &project.cargo_lock_path(),
                            registry_package_path,
                        )
                        .context("Can't check if Cargo.lock dependencies are up to date")?;
                    if are_dependencies_updated {
                        diff.commits.push("chore: update dependencies".to_string());
                    } else {
                        info!("{}: already up to date", package.name);
                    }
                }
                // The local package is identical to the registry one, which means that
                // the package was published at this commit, so we will not count this commit
                // as part of the release.
                // We can process the next create.
                break;
            } else if registry_package.version != package.version {
                info!("{}: the local package has already a different version with respect to the registry package, so release-plz will not update it", package.name);
                diff.set_version_unpublished();
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

/// Check if commit belongs to a previous version of the package.
fn is_commit_too_old(
    repository: &Repo,
    tag_commit: Option<&str>,
    current_commit_hash: &str,
) -> bool {
    if let Some(tag_commit) = tag_commit.as_ref() {
        if repository.is_ancestor(current_commit_hash, tag_commit) {
            debug!("stopping looking at git history because the current commit ({}) is an ancestor of the commit ({}) tagged with the previous version.", current_commit_hash, tag_commit);
            return true;
        }
    }
    false
}

/// Check if release-plz should check the semver compatibility of the package.
/// - `run_semver_check` is true if the user wants to run the semver check.
fn should_check_semver(package: &Package, run_semver_check: bool) -> bool {
    let is_cargo_semver_checks_installed = semver_check::is_cargo_semver_checks_installed;
    run_semver_check && is_library(package) && is_cargo_semver_checks_installed()
}

/// Compare the dependencies of the registry package and the local one.
/// Check if the dependencies of the registry package were updated.
/// This function checks only dependencies of `Cargo.toml`.
fn are_toml_dependencies_updated(
    registry_dependencies: &[Dependency],
    local_dependencies: &[Dependency],
) -> bool {
    local_dependencies
        .iter()
        .any(|d| d.path.is_none() && !registry_dependencies.contains(d))
}

fn workspace_members(manifest: impl AsRef<Path>) -> anyhow::Result<impl Iterator<Item = Package>> {
    let manifest = manifest.as_ref();
    let packages = cargo_utils::workspace_members(Some(manifest))
        .map_err(|e| anyhow!("cannot read workspace members in manifest {manifest:?}: {e}"))?
        .into_iter();
    Ok(packages)
}

pub fn workspace_packages(manifest: impl AsRef<Path>) -> anyhow::Result<Vec<Package>> {
    workspace_members(manifest).map(|members| members.collect())
}

pub fn publishable_packages(manifest: impl AsRef<Path>) -> anyhow::Result<Vec<Package>> {
    workspace_members(manifest).map(|members| members.filter(|p| p.is_publishable()).collect())
}

pub trait Publishable {
    fn is_publishable(&self) -> bool;
}

impl Publishable for Package {
    /// Return true if the field `publish` in Cargo.toml:
    /// - is not `[]`
    /// - is not `false`
    fn is_publishable(&self) -> bool {
        if let Some(publish) = &self.publish {
            // The package can be published only to certain registries
            !publish.is_empty()
        } else {
            // The package can be published anywhere
            true
        }
    }
}

fn is_library(package: &Package) -> bool {
    package
        .targets
        .iter()
        .any(|t| t.kind.contains(&"lib".to_string()))
}

pub fn copy_to_temp_dir(target: &Path) -> anyhow::Result<TempDir> {
    let tmp_dir = tempdir().context("cannot create temporary directory")?;
    dir::copy(target, tmp_dir.as_ref(), &dir::CopyOptions::default()).map_err(|e| {
        anyhow!(
            "cannot copy directory {target:?} to {tmp_dir:?}: {e} Error kind: {:?}",
            e.kind
        )
    })?;
    Ok(tmp_dir)
}

trait PackageDependencies {
    /// Returns the `updated_packages` which should be updated in the dependencies of the package.
    fn dependencies_to_update<'a>(
        &self,
        updated_packages: &'a [(&Package, &Version)],
    ) -> anyhow::Result<Vec<&'a Package>>;
}

impl PackageDependencies for Package {
    fn dependencies_to_update<'a>(
        &self,
        updated_packages: &'a [(&Package, &Version)],
    ) -> anyhow::Result<Vec<&'a Package>> {
        let mut package_manifest = LocalManifest::try_new(self.manifest_path.as_std_path())?;
        let package_dir = manifest_dir(&package_manifest.path)?.to_owned();

        let mut deps_to_update: Vec<&Package> = vec![];
        for (p, next_ver) in updated_packages {
            let canonical_path = p.canonical_path()?;
            let matching_deps = package_manifest
                .get_dependency_tables_mut()
                .flat_map(|t| t.iter_mut().filter_map(|(_, d)| d.as_table_like_mut()))
                .filter(|d| d.contains_key("version"))
                .filter(|d| {
                    let dependency_path = d
                        .get("path")
                        .and_then(|i| i.as_str())
                        .and_then(|relpath| fs::canonicalize(package_dir.join(relpath)).ok());
                    match dependency_path {
                        Some(dep_path) => dep_path == canonical_path,
                        None => false,
                    }
                });

            for dep in matching_deps {
                let old_req = dep
                    .get("version")
                    .expect("filter ensures this")
                    .as_str()
                    .unwrap_or("*");
                if upgrade_requirement(old_req, next_ver)?.is_some() {
                    deps_to_update.push(p);
                }
            }
        }

        Ok(deps_to_update)
    }
}
