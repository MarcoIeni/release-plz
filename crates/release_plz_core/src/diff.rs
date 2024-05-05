use git_cliff_core::commit::Commit;
use regex::Regex;

use crate::semver_check::SemverCheck;

/// Difference between local and registry package (i.e. the last released version)
#[derive(Debug)]
pub(crate) struct Diff<'a> {
    /// List of commits from last released version to last local changes.
    pub commits: Vec<Commit<'a>>,
    /// Whether the package name exists in the registry or not.
    pub registry_package_exists: bool,
    /// Whether the current local version is published to the registry.
    /// If the current version is still unpublished, the package will not be processed.
    pub is_version_published: bool,
    /// Semver incompatible changes.
    pub semver_check: SemverCheck,
}

impl<'a> Diff<'a> {
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
        self.is_version_published = false;
    }

    pub fn set_semver_check(&mut self, semver_check: SemverCheck) {
        self.semver_check = semver_check;
    }

    pub fn add_commits(&mut self, commits: &[Commit<'a>]) {
        for c in commits {
            if !self.commits.contains(c) {
                self.commits.push(c.clone());
            }
        }
    }

    /// Return `true` if any commit message matches the given pattern.
    pub fn any_commit_matches(&self, pattern: &Regex) -> bool {
        self.commits
            .iter()
            .any(|commit| pattern.is_match(&commit.message))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    pub fn create_diff() -> Diff<'static> {
        let mut diff = Diff::new(false);
        diff.add_commits(&vec![Commit::new(
            "1e6903d".to_string(),
            "feature release".to_string(),
        )]);
        diff
    }

    #[test]
    fn test_is_commit_message_matched() {
        let diff = create_diff();
        let pattern = Regex::new(r"^feat").unwrap();
        let present = diff.any_commit_matches(&pattern);
        assert!(present);
    }

    #[test]
    fn test_is_commit_message_not_matched() {
        let diff = create_diff();
        let pattern = Regex::new(r"mismatch").unwrap();
        let present = diff.any_commit_matches(&pattern);
        assert!(!present);
    }
}
