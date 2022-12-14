use semver::Version;

use crate::VersionIncrement;

pub trait NextVersion {
    fn next<I>(&self, commits: I) -> Self
    where
        I: IntoIterator,
        I::Item: AsRef<str>;

    /// Increments the major version number for this Version.
    fn increment_major(&self) -> Self;
    /// Increments the minor version number for this Version.
    fn increment_minor(&self) -> Self;
    /// Increments the patch version number for this Version.
    fn increment_patch(&self) -> Self;
}

impl NextVersion for Version {
    /// Analyze commits and determine the next version based on
    /// [conventional commits](https://www.conventionalcommits.org/) and
    /// [semantic versioning](https://semver.org/):
    /// - If no commits are passed, the version is unchanged.
    /// - If some commits are present, but none of them match conventional commits specification,
    ///   the version is incremented as a Patch.
    /// - If some commits match conventional commits, then the next version is calculated by using
    ///   [these](https://www.conventionalcommits.org/en/v1.0.0/#how-does-this-relate-to-semverare) rules.
    ///
    /// ```rust
    /// use next_version::NextVersion;
    /// use semver::Version;
    ///
    /// let commits = vec!["feat: make coffe"];
    /// let version = Version::new(0, 3, 3);
    /// assert_eq!(version.next(commits), Version::new(0, 3, 4));
    /// ```
    fn next<I>(&self, commits: I) -> Self
    where
        I: IntoIterator,
        I::Item: AsRef<str>,
    {
        let increment = VersionIncrement::from_commits(self, commits);
        match increment {
            Some(increment) => increment.bump(self),
            None => self.clone(),
        }
    }

    // taken from https://github.com/killercup/cargo-edit/blob/643e9253a84db02c52a7fa94f07d786d281362ab/src/version.rs#L38
    fn increment_major(&self) -> Self {
        Self {
            major: self.major + 1,
            minor: 0,
            patch: 0,
            pre: semver::Prerelease::EMPTY,
            build: semver::BuildMetadata::EMPTY,
        }
    }

    // taken from https://github.com/killercup/cargo-edit/blob/643e9253a84db02c52a7fa94f07d786d281362ab/src/version.rs#L46
    fn increment_minor(&self) -> Self {
        Self {
            major: self.major,
            minor: self.minor + 1,
            patch: 0,
            pre: semver::Prerelease::EMPTY,
            build: semver::BuildMetadata::EMPTY,
        }
    }

    // taken from https://github.com/killercup/cargo-edit/blob/643e9253a84db02c52a7fa94f07d786d281362ab/src/version.rs#L53
    fn increment_patch(&self) -> Self {
        Self {
            major: self.major,
            minor: self.minor,
            patch: self.patch + 1,
            pre: semver::Prerelease::EMPTY,
            build: semver::BuildMetadata::EMPTY,
        }
    }
}
