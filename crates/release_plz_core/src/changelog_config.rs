use chrono::NaiveDate;
use git_cliff_core::config::{ChangelogConfig, Config as GitCliffConfig, GitConfig};

#[derive(Debug, Clone)]
pub struct ChangelogRequest {
    /// When the new release is published. If unspecified, current date is used.
    pub release_date: Option<NaiveDate>,
    pub changelog_config: GitCliffConfig,
}

impl Default for ChangelogRequest {
    fn default() -> Self {
        Self {
            release_date: None,
            changelog_config: GitCliffConfig {
                changelog: ChangelogConfig::default(),
                git: GitConfig::default(),
            },
        }
    }
}
