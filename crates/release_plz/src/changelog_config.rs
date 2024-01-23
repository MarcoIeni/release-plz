use anyhow::Context;
use git_cliff_core::config::ChangelogConfig;
use regex::Regex;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default, PartialEq, Eq, Debug, Clone, JsonSchema)]
pub struct ChangelogCfg {
    pub header: Option<String>,
    pub body: Option<String>,
    /// If set to `true`, leading and trailing whitespace are removed from the [`Self::body`].
    pub trim: Option<bool>,
    /// An array of commit preprocessors for manipulating the commit messages before parsing/grouping them.
    pub commit_preprocessors: Option<Vec<TextProcessor>>,
    /// How to sort the commits inside the various sections.
    pub sort_commits: Option<Sorting>,
    /// An array of link parsers for extracting external references, and turning them into URLs, using regex.
    pub link_parsers: Option<Vec<LinkParser>>,
    /// Commits that don't match any of the commit parsers are skipped.
    pub commit_parsers: Option<Vec<CommitParser>>,
    /// Whether to protect all breaking changes from being skipped by a commit parser.
    pub protect_breaking_commits: Option<bool>,
    /// A regular expression for matching the git tags to add to the changelog.
    pub tag_pattern: Option<String>,
}

impl ChangelogCfg {
    pub fn is_default(&self) -> bool {
        let default_config = ChangelogCfg::default();
        &default_config == self
    }
}

/// Used for modifying commit messages.
#[derive(Serialize, Deserialize, Default, PartialEq, Eq, Debug, Clone, JsonSchema)]
pub struct TextProcessor {
    /// Regex for matching a text to replace.
    pub pattern: String,
    /// Replacement text.
    pub replace: Option<String>,
    /// Command that will be run for replacing the commit message.
    pub replace_command: Option<String>,
}

impl TryFrom<TextProcessor> for git_cliff_core::config::TextProcessor {
    fn try_from(cfg: TextProcessor) -> Result<Self, Self::Error> {
        Ok(Self {
            pattern: to_regex(&cfg.pattern, "pattern")?,
            replace: cfg.replace,
            replace_command: cfg.replace_command,
        })
    }

    type Error = anyhow::Error;
}

#[derive(Serialize, Deserialize, Default, PartialEq, Eq, Debug, Clone, Copy, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Sorting {
    Oldest,
    #[default]
    Newest,
}

impl std::fmt::Display for Sorting {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Sorting::Oldest => write!(f, "oldest"),
            Sorting::Newest => write!(f, "newest"),
        }
    }
}

#[derive(Serialize, Deserialize, Default, PartialEq, Eq, Debug, Clone, JsonSchema)]
pub struct LinkParser {
    /// Regex for finding links in the commit message.
    pub pattern: String,
    /// The string used to generate the link URL.
    pub href: String,
    /// The string used to generate the link text.
    pub text: Option<String>,
}

impl TryFrom<LinkParser> for git_cliff_core::config::LinkParser {
    type Error = anyhow::Error;

    fn try_from(value: LinkParser) -> Result<Self, Self::Error> {
        Ok(Self {
            pattern: to_regex(&value.pattern, "pattern")?,
            href: value.href,
            text: value.text,
        })
    }
}

/// Parser for grouping commits.
#[derive(Serialize, Deserialize, Default, PartialEq, Eq, Debug, Clone, JsonSchema)]
pub struct CommitParser {
    /// Regex for matching the commit message.
    pub message: Option<String>,
    /// Regex for matching the commit body.
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

impl TryFrom<CommitParser> for git_cliff_core::config::CommitParser {
    type Error = anyhow::Error;

    fn try_from(cfg: CommitParser) -> Result<Self, Self::Error> {
        Ok(Self {
            message: to_opt_regex(cfg.message.as_deref(), "message")?,
            body: to_opt_regex(cfg.body.as_deref(), "body")?,
            group: cfg.group,
            default_scope: cfg.default_scope,
            scope: cfg.scope,
            skip: cfg.skip,
            field: cfg.field,
            pattern: to_opt_regex(cfg.pattern.as_deref(), "pattern")?,
        })
    }
}

fn to_regex(input: &str, element_name: &str) -> anyhow::Result<Regex> {
    Regex::new(input).with_context(|| format!("failed to parse `{element_name}` regex"))
}

/// Convert an input string to an (optional) regex.
fn to_opt_regex(input: Option<&str>, element_name: &str) -> anyhow::Result<Option<Regex>> {
    input.map(|i| to_regex(i, element_name)).transpose()
}

fn vec_try_into<T, U>(vec: Vec<T>, name: &str) -> anyhow::Result<Vec<U>>
where
    T: TryInto<U, Error = anyhow::Error>,
{
    vec.into_iter()
        .map(|cp| {
            cp.try_into()
                .with_context(|| format!("failed to parse {name}"))
        })
        .collect()
}

impl TryFrom<ChangelogCfg> for git_cliff_core::config::Config {
    type Error = anyhow::Error;

    fn try_from(cfg: ChangelogCfg) -> Result<Self, Self::Error> {
        let commit_preprocessors: Option<Vec<git_cliff_core::config::TextProcessor>> = cfg
            .commit_preprocessors
            .map(|p| vec_try_into(p, "commit_preprocessors"))
            .transpose()?;
        let link_parsers: Option<Vec<git_cliff_core::config::LinkParser>> = cfg
            .link_parsers
            .map(|l| vec_try_into(l, "link_parsers"))
            .transpose()?;
        let tag_pattern = to_opt_regex(cfg.tag_pattern.as_deref(), "tag_pattern")?;

        let sort_commits = cfg.sort_commits.unwrap_or_default();

        let commit_parsers: Option<Vec<git_cliff_core::config::CommitParser>> = cfg
            .commit_parsers
            .map(|c| vec_try_into(c, "commit_parsers"))
            .transpose()?;

        Ok(Self {
            changelog: ChangelogConfig {
                header: cfg.header,
                body: cfg.body,
                trim: cfg.trim,
                postprocessors: None,
                footer: None,
            },
            git: git_cliff_core::config::GitConfig {
                conventional_commits: None,
                filter_unconventional: None,
                split_commits: None,
                commit_preprocessors,
                commit_parsers,
                protect_breaking_commits: cfg.protect_breaking_commits,
                link_parsers,
                filter_commits: None,
                tag_pattern,
                skip_tags: None,
                ignore_tags: None,
                topo_order: None,
                sort_commits: Some(format!("{sort_commits}")),
                limit_commits: None,
            },
        })
    }
}
