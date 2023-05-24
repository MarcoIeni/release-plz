use anyhow::Context;
use chrono::{DateTime, NaiveDate, Utc};
use git_cliff::changelog::Changelog as GitCliffChangelog;
use git_cliff_core::{
    commit::Commit,
    config::{ChangelogConfig, CommitParser, Config, GitConfig},
    release::Release,
};
use regex::Regex;

use crate::changelog_parser;

pub const CHANGELOG_HEADER: &str = r#"# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]
"#;

pub const CHANGELOG_FILENAME: &str = "CHANGELOG.md";

pub struct Changelog<'a> {
    release: Release<'a>,
    config: Option<Config>,
    release_link: Option<String>,
}

impl Changelog<'_> {
    /// Generate the full changelog.
    pub fn generate(self) -> String {
        let config = self
            .config
            .unwrap_or_else(|| default_git_cliff_config(None, self.release_link.as_deref()));
        let changelog = GitCliffChangelog::new(vec![self.release], &config)
            .expect("error while building changelog");
        let mut out = Vec::new();
        changelog
            .generate(&mut out)
            .expect("cannot generate changelog");
        String::from_utf8(out).expect("cannot convert bytes to string")
    }

    /// Update an existing changelog.
    pub fn prepend(self, old_changelog: impl Into<String>) -> anyhow::Result<String> {
        let old_changelog: String = old_changelog.into();
        if let Ok(Some(last_version)) = changelog_parser::last_version_from_str(&old_changelog) {
            let next_version = self
                .release
                .version
                .as_ref()
                .context("current release contains no version")?;
            if next_version == &last_version {
                // The changelog already contains this version, so we don't update the changelog.
                return Ok(old_changelog);
            }
        }
        let old_header = changelog_parser::parse_header(&old_changelog);
        let config = self
            .config
            .unwrap_or_else(|| default_git_cliff_config(old_header, self.release_link.as_deref()));
        let changelog = GitCliffChangelog::new(vec![self.release], &config)
            .context("error while building changelog")?;
        let mut out = Vec::new();
        changelog
            .prepend(old_changelog, &mut out)
            .expect("cannot update changelog");
        String::from_utf8(out).context("cannot convert bytes to string")
    }
}

fn default_git_cliff_config(header: Option<String>, release_link: Option<&str>) -> Config {
    Config {
        changelog: default_changelog_config(header, release_link),
        git: default_git_config(),
    }
}

pub struct ChangelogBuilder {
    commits: Vec<String>,
    version: String,
    config: Option<Config>,
    release_date: Option<NaiveDate>,
    release_link: Option<String>,
}

impl ChangelogBuilder {
    pub fn new(commits: Vec<impl Into<String>>, version: impl Into<String>) -> Self {
        Self {
            commits: commits.into_iter().map(|s| s.into()).collect(),
            version: version.into(),
            config: None,
            release_date: None,
            release_link: None,
        }
    }

    pub fn with_release_date(self, release_date: NaiveDate) -> Self {
        Self {
            release_date: Some(release_date),
            ..self
        }
    }

    pub fn with_release_link(self, release_link: impl Into<String>) -> Self {
        Self {
            release_link: Some(release_link.into()),
            ..self
        }
    }

    pub fn with_config(self, config: Config) -> Self {
        Self {
            config: Some(config),
            ..self
        }
    }

    pub fn build(self) -> Changelog<'static> {
        let git_config = self
            .config
            .clone()
            .map(|c| c.git)
            .unwrap_or_else(default_git_config);
        let release_date = self.release_timestamp();
        let commits = self
            .commits
            .clone()
            .into_iter()
            .map(|c| Commit::new("id".to_string(), c))
            .filter_map(|c| c.process(&git_config).ok())
            .collect();

        Changelog {
            release: Release {
                version: Some(self.version),
                commits,
                commit_id: None,
                timestamp: release_date,
                previous: None,
            },
            release_link: self.release_link,
            config: self.config,
        }
    }

    /// Returns the provided release timestamp, if provided.
    /// Current timestamp otherwise.
    fn release_timestamp(&self) -> i64 {
        self.release_date
            .and_then(|date| date.and_hms_opt(0, 0, 0))
            .map(|d| DateTime::<Utc>::from_utc(d, Utc))
            .unwrap_or_else(Utc::now)
            .timestamp()
    }
}

fn default_git_config() -> GitConfig {
    GitConfig {
        conventional_commits: Some(true),
        filter_unconventional: Some(false),
        commit_parsers: Some(commit_parsers()),
        filter_commits: Some(true),
        tag_pattern: None,
        skip_tags: None,
        split_commits: None,
        protect_breaking_commits: None,
        topo_order: None,
        ignore_tags: None,
        limit_commits: None,
        sort_commits: None,
        commit_preprocessors: None,
        link_parsers: None,
    }
}

/// Commit parsers based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
fn commit_parsers() -> Vec<CommitParser> {
    vec![
        CommitParser {
            message: Regex::new("^feat").ok(),
            body: None,
            group: Some(String::from("added")),
            default_scope: None,
            skip: None,
            scope: None,
        },
        CommitParser {
            message: Regex::new("^changed").ok(),
            body: None,
            group: Some(String::from("changed")),
            default_scope: None,
            skip: None,
            scope: None,
        },
        CommitParser {
            message: Regex::new("^deprecated").ok(),
            body: None,
            group: Some(String::from("deprecated")),
            default_scope: None,
            skip: None,
            scope: None,
        },
        CommitParser {
            message: Regex::new("^removed").ok(),
            body: None,
            group: Some(String::from("removed")),
            default_scope: None,
            skip: None,
            scope: None,
        },
        CommitParser {
            message: Regex::new("^fix").ok(),
            body: None,
            group: Some(String::from("fixed")),
            default_scope: None,
            skip: None,
            scope: None,
        },
        CommitParser {
            message: Regex::new("^security").ok(),
            body: None,
            group: Some(String::from("security")),
            default_scope: None,
            skip: None,
            scope: None,
        },
        CommitParser {
            message: Regex::new(".*").ok(),
            body: None,
            group: Some(String::from("other")),
            default_scope: None,
            skip: None,
            scope: None,
        },
    ]
}

fn default_changelog_config(header: Option<String>, release_link: Option<&str>) -> ChangelogConfig {
    ChangelogConfig {
        header: Some(header.unwrap_or(String::from(CHANGELOG_HEADER))),
        body: Some(default_changelog_body_config(release_link)),
        footer: None,
        trim: Some(true),
    }
}

fn default_changelog_body_config(release_link: Option<&str>) -> String {
    let pre = r#"
    ## [{{ version | trim_start_matches(pat="v") }}]"#;
    let post = r#" - {{ timestamp | date(format="%Y-%m-%d") }}
{% for group, commits in commits | group_by(attribute="group") %}
### {{ group | upper_first }}
{% for commit in commits %}
{%- if commit.scope -%}
- *({{commit.scope}})* {% if commit.breaking %}[**breaking**] {% endif %}{{ commit.message }}{%- if commit.links %} ({% for link in commit.links %}[{{link.text}}]({{link.href}}) {% endfor -%}){% endif %}
{% else -%}
- {% if commit.breaking %}[**breaking**] {% endif %}{{ commit.message }}
{% endif -%}
{% endfor -%}
{% endfor %}"#;

    match release_link {
        Some(link) => format!("{pre}({link}){post}"),
        None => format!("{pre}{post}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn changelog_entries_are_generated() {
        let commits = vec!["fix: myfix", "simple update"];
        let changelog = ChangelogBuilder::new(commits, "1.1.1")
            .with_release_date(NaiveDate::from_ymd_opt(2015, 5, 15).unwrap())
            .build();

        expect_test::expect![[r####"
            # Changelog
            All notable changes to this project will be documented in this file.

            The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
            and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

            ## [Unreleased]

            ## [1.1.1] - 2015-05-15

            ### Fixed
            - myfix

            ### Other
            - simple update
        "####]]
        .assert_eq(&changelog.generate());
    }

    #[test]
    fn changelog_entry_with_link_is_generated() {
        let commits = vec!["fix: myfix"];
        let changelog = ChangelogBuilder::new(commits, "1.1.1")
            .with_release_date(NaiveDate::from_ymd_opt(2015, 5, 15).unwrap())
            .with_release_link("https://github.com/MarcoIeni/release-plz/compare/release-plz-v0.2.24...release-plz-v0.2.25")
            .build();

        expect_test::expect![[r####"
            # Changelog
            All notable changes to this project will be documented in this file.

            The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
            and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

            ## [Unreleased]

            ## [1.1.1](https://github.com/MarcoIeni/release-plz/compare/release-plz-v0.2.24...release-plz-v0.2.25) - 2015-05-15

            ### Fixed
            - myfix
        "####]]
        .assert_eq(&changelog.generate());
    }

    #[test]
    fn generated_changelog_is_updated_correctly() {
        let commits = vec!["fix: myfix", "simple update"];
        let changelog = ChangelogBuilder::new(commits, "1.1.1")
            .with_release_date(NaiveDate::from_ymd_opt(2015, 5, 15).unwrap())
            .build();

        let generated_changelog = changelog.generate();

        let commits = vec!["fix: myfix2", "complex update"];
        let changelog = ChangelogBuilder::new(commits, "1.1.2")
            .with_release_date(NaiveDate::from_ymd_opt(2015, 5, 15).unwrap())
            .build();

        expect_test::expect![[r####"
            # Changelog
            All notable changes to this project will be documented in this file.

            The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
            and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

            ## [Unreleased]

            ## [1.1.2] - 2015-05-15

            ### Fixed
            - myfix2

            ### Other
            - complex update

            ## [1.1.1] - 2015-05-15

            ### Fixed
            - myfix

            ### Other
            - simple update
        "####]]
        .assert_eq(&changelog.prepend(generated_changelog).unwrap());
    }

    #[test]
    fn changelog_is_updated() {
        let commits = vec!["fix: myfix", "simple update"];
        let changelog = ChangelogBuilder::new(commits, "1.1.1")
            .with_release_date(NaiveDate::from_ymd_opt(2015, 5, 15).unwrap())
            .build();
        let old_body = r#"## [1.1.0] - 1970-01-01

### fix bugs
- my awesomefix

### other
- complex update
"#;
        let old = format!("{CHANGELOG_HEADER}\n{old_body}");
        let new = changelog.prepend(old).unwrap();
        expect_test::expect![[r####"
            # Changelog
            All notable changes to this project will be documented in this file.

            The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
            and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

            ## [Unreleased]

            ## [1.1.1] - 2015-05-15

            ### Fixed
            - myfix

            ### Other
            - simple update

            ## [1.1.0] - 1970-01-01

            ### fix bugs
            - my awesomefix

            ### other
            - complex update
        "####]]
        .assert_eq(&new);
    }

    #[test]
    fn changelog_without_header_is_updated() {
        let commits = vec!["fix: myfix", "simple update"];
        let changelog = ChangelogBuilder::new(commits, "1.1.1")
            .with_release_date(NaiveDate::from_ymd_opt(2015, 5, 15).unwrap())
            .build();
        let old = r#"## [1.1.0] - 1970-01-01

### fix bugs
- my awesomefix

### other
- complex update
"#;
        let new = changelog.prepend(old);
        expect_test::expect![[r####"
            # Changelog
            All notable changes to this project will be documented in this file.

            The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
            and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

            ## [Unreleased]

            ## [1.1.1] - 2015-05-15

            ### Fixed
            - myfix

            ### Other
            - simple update
            ## [1.1.0] - 1970-01-01

            ### fix bugs
            - my awesomefix

            ### other
            - complex update
        "####]]
        .assert_eq(&new.unwrap());
    }
}

#[test]
fn empty_changelog_is_updated() {
    let commits = vec!["fix: myfix", "simple update"];
    let changelog = ChangelogBuilder::new(commits, "1.1.1")
        .with_release_date(NaiveDate::from_ymd_opt(2015, 5, 15).unwrap())
        .build();
    let new = changelog.prepend(CHANGELOG_HEADER);
    expect_test::expect![[r####"
        # Changelog
        All notable changes to this project will be documented in this file.

        The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
        and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

        ## [Unreleased]

        ## [1.1.1] - 2015-05-15

        ### Fixed
        - myfix

        ### Other
        - simple update
    "####]]
    .assert_eq(&new.unwrap());
}

#[test]
fn same_version_is_not_added() {
    let commits = vec!["fix: myfix", "simple update"];

    // this version is already in the changelog
    let changelog = ChangelogBuilder::new(commits, "1.1.0")
        .with_release_date(NaiveDate::from_ymd_opt(2015, 5, 15).unwrap())
        .build();

    let old = r#"## [1.1.0] - 1970-01-01

### fix bugs
- my awesomefix

### other
- complex update
"#;
    let new = changelog.prepend(old).unwrap();
    assert_eq!(old, new)
}
