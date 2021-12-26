use cargo_edit::VersionExt;
use conventional_commit_parser::commit::{CommitType, ConventionalCommit};
use semver::Version;

pub enum VersionIncrement {
    Major,
    Minor,
    Patch,
}

impl VersionIncrement {
    /// Analyze commits and determine which part of version to increment based on
    /// [conventional commits](https://www.conventionalcommits.org/) and
    /// [Semantic versioning](https://semver.org/).
    /// - If no commits are present, None is returned, i.e. the version should not be incremented.
    /// - If no conventional commits are present, the version is incremented as a Patch
    pub fn from_commits(current_version: &Version, commits: &[String]) -> Option<Self> {
        if commits.is_empty() {
            None
        } else {
            // Parse commits and keep only the ones that follow conventional commits specification.
            let commits: Vec<ConventionalCommit> = commits
                .iter()
                .filter_map(|c| conventional_commit_parser::parse(c).ok())
                .collect();

            Some(VersionIncrement::from_conventional_commits(
                current_version,
                &commits,
            ))
        }
    }

    /// If no conventional commits are present, the version is incremented as a Patch
    fn from_conventional_commits(
        current_version: &Version,
        commits: &[ConventionalCommit],
    ) -> Self {
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
}

impl VersionIncrement {
    pub fn bump(&self, version: &Version) -> Version {
        let mut new_version = version.clone();
        match self {
            VersionIncrement::Major => new_version.increment_major(),
            VersionIncrement::Minor => new_version.increment_minor(),
            VersionIncrement::Patch => new_version.increment_patch(),
        }
        new_version
    }
}
