use chrono::{Date, TimeZone, Utc};
use git_cliff::changelog::Changelog as GitCliffChangelog;
use git_cliff_core::{
    commit::Commit,
    config::{ChangelogConfig, CommitParser, Config, GitConfig},
    regex::Regex,
    release::Release,
};

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
}

impl Changelog<'_> {
    /// Generate the full changelog.
    pub fn generate(self) -> String {
        let config = self.config.unwrap_or_else(default_git_cliff_config);
        let changelog = GitCliffChangelog::new(vec![self.release], &config)
            .expect("error while building changelog");
        let mut out = Vec::new();
        changelog
            .generate(&mut out)
            .expect("cannot generate changelog");
        String::from_utf8(out).expect("cannot convert bytes to string")
    }

    /// Update an existing changelog.
    pub fn prepend(self, old_changelog: impl Into<String>) -> String {
        let config = self.config.unwrap_or_else(default_git_cliff_config);
        let changelog = GitCliffChangelog::new(vec![self.release], &config)
            .expect("error while building changelog");
        let mut out = Vec::new();
        changelog
            .prepend(old_changelog.into(), &mut out)
            .expect("cannot update changelog");
        String::from_utf8(out).expect("cannot convert bytes to string")
    }
}

fn default_git_cliff_config() -> Config {
    Config {
        changelog: default_changelog_config(),
        git: default_git_config(),
    }
}

pub struct ChangelogBuilder {
    commits: Vec<String>,
    version: String,
    config: Option<Config>,
    release_date: Option<Date<Utc>>,
}

impl ChangelogBuilder {
    pub fn new(commits: Vec<impl Into<String>>, version: impl Into<String>) -> Self {
        Self {
            commits: commits.into_iter().map(|s| s.into()).collect(),
            version: version.into(),
            release_date: None,
            config: None,
        }
    }

    pub fn with_release_date(self, timestamp: Date<Utc>) -> Self {
        Self {
            release_date: Some(timestamp),
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
            config: self.config,
        }
    }

    fn release_timestamp(&self) -> i64 {
        let release_date = self.release_date.unwrap_or_else(|| Utc::now().date());
        let difference = release_date - Utc.ymd(1970, 1, 1);
        difference.num_seconds()
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
        ignore_tags: None,
        date_order: None,
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
        },
        CommitParser {
            message: Regex::new("^changed").ok(),
            body: None,
            group: Some(String::from("changed")),
            default_scope: None,
            skip: None,
        },
        CommitParser {
            message: Regex::new("^deprecated").ok(),
            body: None,
            group: Some(String::from("deprecated")),
            default_scope: None,
            skip: None,
        },
        CommitParser {
            message: Regex::new("^removed").ok(),
            body: None,
            group: Some(String::from("removed")),
            default_scope: None,
            skip: None,
        },
        CommitParser {
            message: Regex::new("^fix").ok(),
            body: None,
            group: Some(String::from("fixed")),
            default_scope: None,
            skip: None,
        },
        CommitParser {
            message: Regex::new("^security").ok(),
            body: None,
            group: Some(String::from("security")),
            default_scope: None,
            skip: None,
        },
        CommitParser {
            message: Regex::new(".*").ok(),
            body: None,
            group: Some(String::from("other")),
            default_scope: None,
            skip: None,
        },
    ]
}

fn default_changelog_config() -> ChangelogConfig {
    ChangelogConfig {
        header: Some(String::from(CHANGELOG_HEADER)),
        body: Some(String::from(
            r#"
## [{{ version | trim_start_matches(pat="v") }}] - {{ timestamp | date(format="%Y-%m-%d") }}
{% for group, commits in commits | group_by(attribute="group") %}
### {{ group | upper_first }}
{% for commit in commits %}
{%- if commit.scope -%}
- *({{commit.scope}})* {% if commit.breaking %}[**breaking**] {% endif %}{{ commit.message }}{%- if commit.links %} ({% for link in commit.links %}[{{link.text}}]({{link.href}}) {% endfor -%}){% endif %}
{% else -%}
- {% if commit.breaking %}[**breaking**] {% endif %}{{ commit.message }}
{% endif -%}
{% endfor -%}
{% endfor %}"#,
        )),
        footer: None,
        trim: Some(true),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn changelog_entries_are_generated() {
        let commits = vec!["fix: myfix", "simple update"];
        let changelog = ChangelogBuilder::new(commits, "1.1.1")
            .with_release_date(Utc.ymd(2015, 5, 15))
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
    fn generated_changelog_is_updated_correctly() {
        let commits = vec!["fix: myfix", "simple update"];
        let changelog = ChangelogBuilder::new(commits, "1.1.1")
            .with_release_date(Utc.ymd(2015, 5, 15))
            .build();

        let generated_changelog = changelog.generate();

        let commits = vec!["fix: myfix2", "complex update"];
        let changelog = ChangelogBuilder::new(commits, "1.1.2")
            .with_release_date(Utc.ymd(2015, 5, 15))
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
        .assert_eq(&changelog.prepend(generated_changelog));
    }

    #[test]
    fn changelog_is_updated() {
        let commits = vec!["fix: myfix", "simple update"];
        let changelog = ChangelogBuilder::new(commits, "1.1.1")
            .with_release_date(Utc.ymd(2015, 5, 15))
            .build();
        let old_body = r#"## [1.1.0] - 1970-01-01

### fix bugs
- my awesomefix

### other
- complex update
"#;
        let old = format!("{CHANGELOG_HEADER}\n{old_body}");
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
        .assert_eq(&new);
    }

    #[test]
    fn changelog_without_header_is_updated() {
        let commits = vec!["fix: myfix", "simple update"];
        let changelog = ChangelogBuilder::new(commits, "1.1.1")
            .with_release_date(Utc.ymd(2015, 5, 15))
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
        .assert_eq(&new);
    }
}

#[test]
fn empty_changelog_is_updated() {
    let commits = vec!["fix: myfix", "simple update"];
    let changelog = ChangelogBuilder::new(commits, "1.1.1")
        .with_release_date(Utc.ymd(2015, 5, 15))
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
    .assert_eq(&new);
}
