/// Difference between local and registry package (i.e. the last released version)
#[derive(Debug)]
pub(crate) struct Diff {
    /// List of commits from last released vesion to last local changes
    pub commits: Vec<String>,
    /// Whether the package name exists in the registry package or not
    pub registry_package_exists: bool,
}

impl Diff {
    pub fn new(registry_package_exists: bool) -> Self {
        Self {
            commits: vec![],
            registry_package_exists,
        }
    }

    pub fn should_update_version(&self) -> bool {
        self.registry_package_exists && !self.commits.is_empty()
    }
}
