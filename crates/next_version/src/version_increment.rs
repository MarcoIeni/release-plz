use conventional_commit_parser::commit::{CommitType, ConventionalCommit};
use semver::Version;

use crate::{NextVersion, VersionUpdater};

#[derive(Debug, PartialEq, Eq)]
pub enum VersionIncrement {
    Major,
    Minor,
    Patch,
    Prerelease,
}

impl VersionIncrement {
    /// Analyze commits and determine which part of version to increment based on
    /// [conventional commits](https://www.conventionalcommits.org/) and
    /// [Semantic versioning](https://semver.org/).
    /// - If no commits are present, [`Option::None`] is returned, because the version should not be incremented.
    /// - If some commits are present and [`semver::Prerelease`] is not empty, the version increment is
    ///   [`VersionIncrement::Prerelease`].
    /// - If some commits are present, but none of them match conventional commits specification,
    ///   the version increment is [`VersionIncrement::Patch`].
    /// - If some commits match conventional commits, then the next version is calculated by using
    ///   [these](https://www.conventionalcommits.org/en/v1.0.0/#how-does-this-relate-to-semverare) rules.
    pub fn from_commits<I>(
        updater: &VersionUpdater,
        current_version: &Version,
        commits: I,
    ) -> Option<Self>
    where
        I: IntoIterator,
        I::Item: AsRef<str>,
    {
        let mut commits = commits.into_iter().peekable();
        let are_commits_present = commits.peek().is_some();
        if !are_commits_present {
            None
        } else {
            if !current_version.pre.is_empty() {
                return Some(VersionIncrement::Prerelease);
            }
            // Parse commits and keep only the ones that follow conventional commits specification.
            let commits: Vec<ConventionalCommit> = commits
                .filter_map(|c| conventional_commit_parser::parse(c.as_ref()).ok())
                .collect();

            Some(Self::from_conventional_commits(
                current_version,
                &commits,
                updater,
            ))
        }
    }

    /// Increments the version to take into account breaking changes.
    /// ```rust
    /// use next_version::VersionIncrement;
    /// use semver::Version;
    ///
    /// let increment = VersionIncrement::breaking(&Version::new(0, 3, 3));
    /// assert_eq!(increment, VersionIncrement::Minor);
    ///
    /// let increment = VersionIncrement::breaking(&Version::new(1, 3, 3));
    /// assert_eq!(increment, VersionIncrement::Major);
    ///
    /// let increment = VersionIncrement::breaking(&Version::parse("1.3.3-alpha.1").unwrap());
    /// assert_eq!(increment, VersionIncrement::Prerelease);
    /// ```
    pub fn breaking(current_version: &Version) -> Self {
        if !current_version.pre.is_empty() {
            Self::Prerelease
        } else if current_version.major == 0 && current_version.minor == 0 {
            Self::Patch
        } else if current_version.major == 0 {
            Self::Minor
        } else {
            Self::Major
        }
    }

    /// If no conventional commits are present, the version is incremented as a Patch
    fn from_conventional_commits(
        current: &Version,
        commits: &[ConventionalCommit],
        updater: &VersionUpdater,
    ) -> Self {
        let is_there_a_feature = || {
            commits
                .iter()
                .any(|commit| commit.commit_type == CommitType::Feature)
        };

        let is_there_a_breaking_change = || commits.iter().any(|commit| commit.is_breaking_change);

        let is_major_bump = || {
            is_there_a_breaking_change()
                && (current.major != 0 || updater.breaking_always_increment_major)
        };

        let is_minor_bump = || {
            if updater.features_always_increment_minor {
                is_there_a_feature() || (current.minor != 0 && is_there_a_breaking_change())
            } else {
                (current.major != 0 && is_there_a_feature())
                    || (current.major == 0 && current.minor != 0 && is_there_a_breaking_change())
            }
        };

        if is_major_bump() {
            Self::Major
        } else if is_minor_bump() {
            Self::Minor
        } else {
            Self::Patch
        }
    }
}

impl VersionIncrement {
    pub fn bump(&self, version: &Version) -> Version {
        match self {
            Self::Major => version.increment_major(),
            Self::Minor => version.increment_minor(),
            Self::Patch => version.increment_patch(),
            Self::Prerelease => version.increment_prerelease(),
        }
    }
}
