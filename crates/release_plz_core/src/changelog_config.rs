use chrono::NaiveDate;
use git_cliff_core::config::{CommitParser, LinkParser, TextProcessor};

use crate::kac_commit_parsers;

#[derive(Debug, Clone, Default)]
pub struct ChangelogRequest {
    /// When the new release is published. If unspecified, current date is used.
    pub release_date: Option<NaiveDate>,
    pub changelog_config: Option<ChangelogConfig>,
}

#[derive(Debug, Clone)]
pub struct ChangelogConfig {
    pub header: Option<String>,
    pub body: Option<String>,
    pub trim: bool,
    pub commit_preprocessors: Vec<TextProcessor>,
    pub sort_commits: Sorting,
    pub link_parsers: Vec<LinkParser>,
    /// Commits that don't match any of the commit parsers are skipped.
    pub commit_parsers: Vec<CommitParser>,
    /// Whether to protect all breaking changes from being skipped by a commit
    /// parser.
    pub protect_breaking_commits: bool,
    pub tag_pattern: Option<String>,
}

impl Default for ChangelogConfig {
    fn default() -> Self {
        Self {
            header: None,
            body: None,
            trim: true,
            commit_preprocessors: vec![],
            sort_commits: Sorting::default(),
            link_parsers: vec![],
            commit_parsers: kac_commit_parsers(),
            protect_breaking_commits: false,
            tag_pattern: None,
        }
    }
}

#[derive(Default, PartialEq, Eq, Debug, Clone, Copy)]
pub enum Sorting {
    Oldest,
    #[default]
    Newest,
}
