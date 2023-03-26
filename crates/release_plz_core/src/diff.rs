use crate::semver_check::SemverCheck;

/// Difference between local and registry package (i.e. the last released version)
#[derive(Debug)]
pub(crate) struct Diff {
    /// List of commits from last released version to last local changes.
    pub commits: Vec<String>,
    /// Whether the package name exists in the registry or not.
    pub registry_package_exists: bool,
    /// Whether the current local version is published to the registry.
    /// If the current version is still unpublished, the package will not be processed.
    pub is_version_published: bool,
    /// Semver incompatible changes.
    pub semver_check: SemverCheck,
}

impl Diff {
    pub fn new(registry_package_exists: bool) -> Self {
        Self {
            commits: vec![],
            registry_package_exists,
            is_version_published: true,
            semver_check: SemverCheck::Skipped,
        }
    }

    pub fn should_update_version(&self) -> bool {
        self.registry_package_exists && !self.commits.is_empty()
    }

    pub fn set_version_unpublished(&mut self) {
        self.is_version_published = false
    }

    pub fn set_semver_check(&mut self, semver_check: SemverCheck) {
        self.semver_check = semver_check
    }
}
