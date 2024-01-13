use chrono::NaiveDate;
use git_cliff_core::config::{CommitParser, Config as GitCliffConfig, LinkParser, TextProcessor};

use crate::kac_commit_parsers;

#[derive(Debug, Clone, Default)]
pub struct ChangelogRequest {
    /// When the new release is published. If unspecified, current date is used.
    pub release_date: Option<NaiveDate>,
    pub changelog_config: Option<GitCliffConfig>,
}
