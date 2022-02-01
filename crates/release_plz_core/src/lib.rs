mod download;
mod release_pr;
mod update;
mod version;

pub use release_pr::*;
pub use update::*;

/// Difference between local and remote package (i.e. the last released version)
#[derive(Debug)]
struct Diff {
    /// List of commits from last released vesion to last local changes
    pub commits: Vec<String>,
    /// Whether the package name exists in the remote package or not
    pub remote_package_exists: bool,
}

impl Diff {
    fn new(remote_package_exists: bool) -> Self {
        Self {
            commits: vec![],
            remote_package_exists,
        }
    }

    fn should_update_version(&self) -> bool {
        self.remote_package_exists && !self.commits.is_empty()
    }
}
