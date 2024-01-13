use git_cliff_core::config::{CommitParser, LinkParser, TextProcessor};

use crate::{kac_commit_parsers, CHANGELOG_HEADER};

#[derive(Clone)]
pub struct ChangelogConfig {
    pub header: String,
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
            header: CHANGELOG_HEADER.to_string(),
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

