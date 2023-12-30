use semver::Version;

use crate::VersionIncrement;

/// Represents a version updater configuration.
///
/// This struct allows controlling version increments based on certain settings.
///
// # Example
///
/// ```
/// use next_version::VersionUpdater;
/// use semver::Version;
///
/// let updated_version = VersionUpdater::new()
///     .with_features_always_increment_minor(false)
///     .with_breaking_always_increment_major(true)
///     .next(&Version::new(1, 2, 3), vec!["feat: commit 1", "fix: commit 2"]);
///
/// assert_eq!(Version::new(1, 3, 0), updated_version);
/// ```
#[derive(Debug)]
pub struct VersionUpdater {
    pub(crate) features_always_increment_minor: bool,
    pub(crate) breaking_always_increment_major: bool,
}

impl Default for VersionUpdater {
    fn default() -> Self {
        Self::new()
    }
}

impl VersionUpdater {
    /// Constructs a new instance with default settings.
    ///
    /// Both minor version increments for feature changes and initial major version
    /// increments for breaking changes are disabled (set to `false`).
    pub fn new() -> Self {
        Self {
            features_always_increment_minor: false,
            breaking_always_increment_major: false,
        }
    }

    /// Configures automatic minor version increments for feature changes.
    ///
    /// When `true` is passed, it enables automatic minor version increments for feature changes.
    /// This means that any introduced feature will trigger a minor version update.
    pub fn with_features_always_increment_minor(
        mut self,
        features_always_increment_minor: bool,
    ) -> Self {
        self.features_always_increment_minor = features_always_increment_minor;
        self
    }

    /// Configures initial major version increments for breaking changes.
    ///
    /// When `true` is passed, it enables the initial major version increment
    /// for breaking changes. This implies that the transition from version 0 to 1
    /// will be triggered by a breaking change in the API.
    pub fn with_breaking_always_increment_major(
        mut self,
        breaking_always_increment_major: bool,
    ) -> Self {
        self.breaking_always_increment_major = breaking_always_increment_major;
        self
    }

    /// Analyze commits and determine the next version.
    pub fn increment<I>(self, version: &Version, commits: I) -> Version
    where
        I: IntoIterator,
        I::Item: AsRef<str>,
    {
        let increment = VersionIncrement::from_commits_with_updater(&self, version, commits);
        match increment {
            Some(increment) => increment.bump(version),
            None => version.clone(),
        }
    }
}
