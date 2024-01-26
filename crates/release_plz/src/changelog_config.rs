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

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone, Copy, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Sorting {
    Oldest,
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

fn to_opt_vec<T, U>(vec: Option<Vec<T>>, element_name: &str) -> anyhow::Result<Option<Vec<U>>>
where
    T: TryInto<U, Error = anyhow::Error>,
{
    vec.map(|v| vec_try_into(v, element_name)).transpose()
}

fn vec_try_into<T, U>(vec: Vec<T>, element_name: &str) -> anyhow::Result<Vec<U>>
where
    T: TryInto<U, Error = anyhow::Error>,
{
    vec.into_iter()
        .map(|cp| {
            cp.try_into()
                .with_context(|| format!("failed to parse {element_name}"))
        })
        .collect()
}

impl TryFrom<ChangelogCfg> for git_cliff_core::config::Config {
    type Error = anyhow::Error;

    fn try_from(cfg: ChangelogCfg) -> Result<Self, Self::Error> {
        let commit_preprocessors: Option<Vec<git_cliff_core::config::TextProcessor>> =
            to_opt_vec(cfg.commit_preprocessors, "commit_preprocessors")?;
        let link_parsers: Option<Vec<git_cliff_core::config::LinkParser>> =
            to_opt_vec(cfg.link_parsers, "link_parsers")?;
        let tag_pattern = to_opt_regex(cfg.tag_pattern.as_deref(), "tag_pattern")?;

        let sort_commits = cfg.sort_commits.map(|s| format!("{s}"));

        let commit_parsers: Option<Vec<git_cliff_core::config::CommitParser>> =
            to_opt_vec(cfg.commit_parsers, "commit_parsers")?;

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
                sort_commits,
                limit_commits: None,
            },
        })
    }
}

// write test to check that the configuration is deserialized correctly
#[cfg(test)]
mod tests {
    #[test]
    fn test_deserialize_toml() {
        let config = git_cliff_core::config::Config {
            changelog: git_cliff_core::config::ChangelogConfig {
                header: Some("Changelog".to_string()),
                body: Some("Body".to_string()),
                trim: Some(true),
                postprocessors: None,
                footer: None,
            },
            git: git_cliff_core::config::GitConfig {
                conventional_commits: Some(true),
                filter_unconventional: Some(true),
                split_commits: Some(true),
                commit_preprocessors: Some(vec![
                    git_cliff_core::config::TextProcessor {
                        pattern: regex::Regex::new("pattern").unwrap(),
                        replace: Some("replace".to_string()),
                        replace_command: Some("replace_command".to_string()),
                    },
                    git_cliff_core::config::TextProcessor {
                        pattern: regex::Regex::new("pattern2").unwrap(),
                        replace: Some("replace2".to_string()),
                        replace_command: Some("replace_command2".to_string()),
                    },
                ]),
                commit_parsers: Some(vec![
                    git_cliff_core::config::CommitParser {
                        message: Some(regex::Regex::new("message").unwrap()),
                        body: Some(regex::Regex::new("body").unwrap()),
                        group: Some("group".to_string()),
                        default_scope: Some("default_scope".to_string()),
                        scope: Some("scope".to_string()),
                        skip: Some(true),
                        field: Some("field".to_string()),
                        pattern: Some(regex::Regex::new("pattern").unwrap()),
                    },
                    git_cliff_core::config::CommitParser {
                        message: Some(regex::Regex::new("message2").unwrap()),
                        body: Some(regex::Regex::new("body2").unwrap()),
                        group: Some("group2".to_string()),
                        default_scope: Some("default_scope2".to_string()),
                        scope: Some("scope2".to_string()),
                        skip: Some(false),
                        field: Some("field2".to_string()),
                        pattern: Some(regex::Regex::new("pattern2").unwrap()),
                    },
                ]),
                protect_breaking_commits: Some(true),
                link_parsers: Some(vec![git_cliff_core::config::LinkParser {
                    pattern: regex::Regex::new("pattern").unwrap(),
                    href: "href".to_string(),
                    text: Some("text".to_string()),
                }]),
                filter_commits: None,
                tag_pattern: None,
                skip_tags: None,
                ignore_tags: None,
                topo_order: None,
                sort_commits: None,
                limit_commits: None,
            },
        };
        expect_test::expect![[r#"
            [changelog]
            header = "Changelog"
            body = "Body"
            trim = true

            [git]
            conventional_commits = true
            filter_unconventional = true
            split_commits = true
            protect_breaking_commits = true

            [[git.commit_preprocessors]]
            pattern = "pattern"
            replace = "replace"
            replace_command = "replace_command"

            [[git.commit_preprocessors]]
            pattern = "pattern2"
            replace = "replace2"
            replace_command = "replace_command2"

            [[git.commit_parsers]]
            message = "message"
            body = "body"
            group = "group"
            default_scope = "default_scope"
            scope = "scope"
            skip = true
            field = "field"
            pattern = "pattern"

            [[git.commit_parsers]]
            message = "message2"
            body = "body2"
            group = "group2"
            default_scope = "default_scope2"
            scope = "scope2"
            skip = false
            field = "field2"
            pattern = "pattern2"

            [[git.link_parsers]]
            pattern = "pattern"
            href = "href"
            text = "text"
        "#]]
        .assert_eq(&toml::to_string(&config).unwrap());
    }
}
