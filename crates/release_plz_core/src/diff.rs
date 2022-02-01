/// Difference between local and remote package (i.e. the last released version)
#[derive(Debug)]
pub(crate) struct Diff {
    /// List of commits from last released vesion to last local changes
    pub commits: Vec<String>,
    /// Whether the package name exists in the remote package or not
    pub remote_package_exists: bool,
}

impl Diff {
    pub fn new(remote_package_exists: bool) -> Self {
        Self {
            commits: vec![],
            remote_package_exists,
        }
    }

    pub fn should_update_version(&self) -> bool {
        self.remote_package_exists && !self.commits.is_empty()
    }
}
