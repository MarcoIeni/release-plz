use cargo_edit::VersionExt;
use cargo_metadata::Version;
use conventional_commit_parser::commit::{CommitType, ConventionalCommit};

use crate::Diff;

pub trait NextVersion {
    /// Analyze commits and determine which part of version to increment based on
    /// [conventional commits](https://www.conventionalcommits.org/)
    fn next(self, diff: &Diff) -> Self;
}

impl NextVersion for Version {
    fn next(mut self, diff: &Diff) -> Self {
        if !diff.remote_crate_exists {
            self
        } else {
            let increment = get_increment_from_commits(&self, &diff.commits);
            increment.bump(&self)
        }
    }
}

fn get_increment_from_commits(current_version: &Version, commits: &[String]) -> VersionIncrement {
    let commits: Vec<ConventionalCommit> = commits
        .iter()
        .filter_map(|c| conventional_commit_parser::parse(c).ok())
        .collect();

    version_increment_from_commit_history(current_version, &commits)
}

// taken from [cocogitto](https://github.com/cocogitto/cocogitto/blob/3a3249cd0167adc5183f2a384155d5e1120a500d/src/conventional/version.rs#L44)
fn version_increment_from_commit_history(
    current_version: &Version,
    commits: &[ConventionalCommit],
) -> VersionIncrement {
    let is_major_bump =
        || current_version.major != 0 && commits.iter().any(|commit| commit.is_breaking_change);

    let is_minor_bump = || {
        commits
            .iter()
            .any(|commit| commit.commit_type == CommitType::Feature)
    };

    if is_major_bump() {
        VersionIncrement::Major
    } else if is_minor_bump() {
        VersionIncrement::Minor
    } else {
        VersionIncrement::Patch
    }
}

enum VersionIncrement {
    Major,
    Minor,
    Patch,
}

impl VersionIncrement {
    fn bump(&self, version: &Version) -> Version {
        let mut new_version = version.clone();
        match self {
            VersionIncrement::Major => new_version.increment_major(),
            VersionIncrement::Minor => new_version.increment_minor(),
            VersionIncrement::Patch => new_version.increment_patch(),
        }
        new_version
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
        assert_eq!(version.clone().next(&diff), version);
    }

    #[test]
    fn commit_without_semver_prefix_increments_patch_version() {
        let diff = Diff {
            remote_crate_exists: true,
            commits: vec!["my change".to_string()],
        };
        let version = Version::new(1, 2, 3);
        assert_eq!(version.next(&diff), Version::new(1, 2, 4));
    }
}
