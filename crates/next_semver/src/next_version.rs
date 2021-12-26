use semver::Version;

use crate::VersionIncrement;

pub trait NextVersion {
    /// Analyze commits and determine the next version based on
    /// [conventional commits](https://www.conventionalcommits.org/) and
    /// [Semantic versioning](https://semver.org/).
    /// - If no commits are passed, the version is unchanged.
    /// - If no conventional commits are present, the version is incremented as a Patch.
    fn next(&self, commits: &[String]) -> Self;
}

impl NextVersion for Version {
    fn next(&self, commits: &[String]) -> Self {
        let increment = VersionIncrement::from_commits(self, commits);
        match increment {
            Some(increment) => increment.bump(self),
            None => self.clone(),
        }
    }
}

#[test]
fn commit_without_semver_prefix_increments_patch_version() {
    let commits = vec!["my change".to_string()];
    let version = Version::new(1, 2, 3);
    assert_eq!(version.next(&commits), Version::new(1, 2, 4));
}
