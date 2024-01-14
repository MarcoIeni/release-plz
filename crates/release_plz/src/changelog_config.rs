use anyhow::Context;
use git_cliff_core::config::ChangelogConfig;
use regex::Regex;
use release_plz_core::kac_commit_parsers;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default, PartialEq, Eq, Debug, Clone, JsonSchema)]
pub struct ChangelogCfg {
    pub header: Option<String>,
    pub body: Option<String>,
    pub trim: Option<bool>,
    #[serde(default)]
    pub commit_preprocessors: Vec<TextProcessor>,
    pub sort_commits: Option<Sorting>,
    #[serde(default)]
    pub link_parsers: Vec<LinkParser>,
    /// Commits that don't match any of the commit parsers are skipped.
    #[serde(default)]
    pub commit_parsers: Vec<CommitParser>,
    /// Whether to protect all breaking changes from being skipped by a commit
    /// parser.
    pub protect_breaking_commits: Option<bool>,
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
            pattern: Regex::new(&cfg.pattern).context("failed to parse regex")?,
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
            pattern: Regex::new(&value.pattern).context("failed to parse regex")?,
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
            message: cfg
                .message
                .map(|m| Regex::new(&m).context("failed to parse message regex"))
                .transpose()?,
            body: cfg
                .body
                .map(|b| Regex::new(&b).context("failed to parse body regex"))
                .transpose()?,
            group: cfg.group,
            default_scope: cfg.default_scope,
            scope: cfg.scope,
            skip: cfg.skip,
            field: cfg.field,
            pattern: cfg
                .pattern
                .map(|p| Regex::new(&p).context("failed to parse pattern regex"))
                .transpose()?,
        })
    }
}

fn vec_try_into<T, U, E>(vec: Vec<T>) -> Result<Vec<U>, E>
where
    T: TryInto<U, Error = E>,
{
    vec.into_iter().map(|cp| cp.try_into()).collect()
}

impl TryFrom<ChangelogCfg> for git_cliff_core::config::Config {
    type Error = anyhow::Error;

    fn try_from(cfg: ChangelogCfg) -> Result<Self, Self::Error> {
        let commit_preprocessors: Vec<git_cliff_core::config::TextProcessor> =
            vec_try_into(cfg.commit_preprocessors)
                .context("failed to parse commit_preprocessors")?;
        let link_parsers: Vec<git_cliff_core::config::LinkParser> =
            vec_try_into(cfg.link_parsers).context("failed to parse link_parsers")?;
        let tag_pattern = cfg
            .tag_pattern
            .map(|pattern| Regex::new(&pattern).context("failed to parse message tag_pattern"))
            .transpose()?;

        let trim = cfg.trim.unwrap_or(true);
        let sort_commits = cfg.sort_commits.unwrap_or(Sorting::default());

        let commit_parsers: Vec<git_cliff_core::config::CommitParser> =
            vec_try_into(cfg.commit_parsers)
                .context("failed to parse commit_parsers")
                .unwrap_or(kac_commit_parsers());

        Ok(Self {
            changelog: ChangelogConfig {
                header: cfg.header,
                body: cfg.body,
                trim: Some(trim),
                postprocessors: None,
                footer: None,
            },
            git: git_cliff_core::config::GitConfig {
                conventional_commits: None,
                filter_unconventional: None,
                split_commits: None,
                commit_preprocessors: Some(commit_preprocessors),
                commit_parsers: Some(commit_parsers),
                protect_breaking_commits: cfg.protect_breaking_commits,
                link_parsers: Some(link_parsers),
                filter_commits: Some(true),
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
