/// Difference between local and registry package (i.e. the last released version)
#[derive(Debug)]
pub(crate) struct Diff {
    /// List of commits from last released vesion to last local changes.
    pub commits: Vec<String>,
    /// Whether the package name exists in the registry or not.
    pub registry_package_exists: bool,
    /// Whether the current local version is published to the registry.
    /// If the current is unpublished, the version will not be updated.
    pub is_version_published: bool,
}

impl Diff {
    pub fn new(registry_package_exists: bool) -> Self {
        Self {
            commits: vec![],
            registry_package_exists,
            is_version_published: true,
        }
    }

    pub fn should_update_version(&self) -> bool {
        self.registry_package_exists && !self.commits.is_empty()
    }

    pub fn set_version_unpublished(&mut self) {
        self.is_version_published = false
    }
}
