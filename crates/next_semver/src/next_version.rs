use semver::Version;

use crate::VersionIncrement;

pub trait NextVersion {
    fn next<I>(&self, commits: I) -> Self
    where
        I: IntoIterator,
        I::Item: AsRef<str>;
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
    /// use next_semver::NextVersion;
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
}
