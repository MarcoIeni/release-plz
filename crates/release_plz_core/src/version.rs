use cargo_metadata::semver::Version;
use next_version::{NextVersion, VersionIncrement};

use crate::diff::Diff;

pub(crate) trait NextVersionFromDiff {
    /// Analyze commits and determine which part of version to increment based on
    /// [conventional commits](https://www.conventionalcommits.org/)
    fn next_from_diff(&self, diff: &Diff) -> Self;
}

impl NextVersionFromDiff for Version {
    fn next_from_diff(&self, diff: &Diff) -> Self {
        if !diff.should_update_version() {
            self.clone()
        } else if diff.incompatibilities.is_some() {
            let increment = VersionIncrement::breaking(self);
            increment.bump(self)
        } else {
            self.next(&diff.commits)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn next_version_of_new_package_is_unchanged() {
        let registry_package_exists = false;
        let diff = Diff::new(registry_package_exists);
        let version = Version::new(1, 2, 3);
        assert_eq!(version.clone().next_from_diff(&diff), version);
    }

    #[test]
    fn next_version_of_existing_package_is_updated() {
        let diff = Diff {
            registry_package_exists: true,
            commits: vec!["my change".to_string()],
            is_version_published: true,
            incompatibilities: None,
        };
        let version = Version::new(1, 2, 3);
        assert_eq!(version.next_from_diff(&diff), Version::new(1, 2, 4));
    }
}
