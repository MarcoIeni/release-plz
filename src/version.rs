use cargo_edit::VersionExt;
use cargo_metadata::Version;

use crate::Diff;

trait NextVersion {
    fn next(self, diff: Diff) -> Self;
}

impl NextVersion for Version {
    fn next(mut self, diff: Diff) -> Self {
        if !diff.remote_crate_exists {
            self
        } else {
            self.increment_patch();
            self
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn next_version_of_new_package_is_unchanged() {
        let remote_crate_exists = false;
        let diff = Diff::new(remote_crate_exists);
        let version = Version::new(1, 2, 3);
        assert_eq!(version.clone().next(diff), version);
    }

    #[test]
    fn commit_without_semver_prefix_increments_patch_version() {
        let diff = Diff {
            remote_crate_exists: true,
            commits: vec!["my change".to_string()],
        };
        let version = Version::new(1, 2, 3);
        assert_eq!(version.next(diff), Version::new(1, 2, 4));
    }
}
