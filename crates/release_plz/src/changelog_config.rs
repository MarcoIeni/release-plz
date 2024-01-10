use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default, PartialEq, Eq, Debug, JsonSchema)]
pub struct ChangelogCfg {
    pub header: Option<String>,
    pub body: Option<String>,
    pub trim: Option<bool>,
    pub commit_preprocessors: Option<Vec<TextProcessor>>,
    pub sort_commits: Option<Sorting>,
    pub link_parsers: Option<Vec<LinkParser>>,
    /// Commits that don't match any of the commit parsers are skipped.
    pub commit_parsers: Option<Vec<CommitParser>>,
    /// Whether to protect all breaking changes from being skipped by a commit
    /// parser.
    pub protect_breaking_commits: Option<bool>,
    pub tag_pattern: Option<String>,
}

/// Used for modifying commit messages.
#[derive(Serialize, Deserialize, Default, PartialEq, Eq, Debug, JsonSchema)]
pub struct TextProcessor {
    /// Regex for matching a text to replace.
    pub pattern: String,
    /// Replacement text.
    pub replace: Option<String>,
    /// Command that will be run for replacing the commit message.
    pub replace_command: Option<String>,
}

#[derive(Serialize, Deserialize, Default, PartialEq, Eq, Debug, Clone, Copy, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Sorting {
    Oldest,
    #[default]
    Newest,
}

#[derive(Serialize, Deserialize, Default, PartialEq, Eq, Debug, JsonSchema)]
pub struct LinkParser {
    /// Regex for finding links in the commit message.
    pub pattern: String,
    /// The string used to generate the link URL.
    pub href: String,
    /// The string used to generate the link text.
    pub text: Option<String>,
}

/// Parser for grouping commits.
#[derive(Serialize, Deserialize, Default, PartialEq, Eq, Debug, JsonSchema)]
pub struct CommitParser {
    /// Regex for matching the commit message.
    pub message: Option<String>,
    pub body: Option<String>,
    /// Group of the commit.
    pub group: Option<String>,
    /// Default scope of the commit.
    pub default_scope: Option<String>,
    /// Commit scope for overriding the default scope.
    pub scope: Option<String>,
    /// Whether to skip this commit group.
    pub skip: Option<bool>,
    /// Field name of the commit to match the regex against.
    pub field: Option<String>,
    /// Regex for matching the field value.
    pub pattern: Option<String>,
}
