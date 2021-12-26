use semver::Version;

use crate::VersionIncrement;

pub trait NextVersion {
    /// Analyze commits and determine the next version based on
    /// [conventional commits](https://www.conventionalcommits.org/) and
    /// [Semantic versioning](https://semver.org/).
    /// - If no commits are passed, the version is unchanged.
    /// - If no conventional commits are present, the version is incremented as a Patch.
    fn next<I>(&self, commits: I) -> Self
    where
        I: IntoIterator,
        I::Item: AsRef<str>;
}

impl NextVersion for Version {
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
