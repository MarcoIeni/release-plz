use chrono::{Date, TimeZone, Utc};
use git_cliff_core::{
    commit::Commit,
    config::{ChangelogConfig, CommitParser, GitConfig, LinkParser},
    regex::Regex,
    release::Release,
    template::Template,
};

pub const CHANGELOG_HEADER: &str = r#"# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]
"#;

pub const CHANGELOG_FILENAME: &str = "CHANGELOG.md";

pub struct ChangelogBuilder {
    commits: Vec<String>,
    version: String,
    release_date: Option<Date<Utc>>,
}

impl ChangelogBuilder {
    pub fn new(commits: Vec<impl Into<String>>, version: impl Into<String>) -> Self {
        Self {
            commits: commits.into_iter().map(|s| s.into()).collect(),
            version: version.into(),
            release_date: None,
        }
    }

    pub fn with_release_date(self, timestamp: Date<Utc>) -> Self {
        Self {
            release_date: Some(timestamp),
            ..self
        }
    }

    pub fn build(self) -> Changelog<'static> {
        let git_config = GitConfig {
            conventional_commits: Some(true),
            filter_unconventional: Some(false),
            commit_parsers: Some(commit_parsers()),
            filter_commits: Some(true),
            tag_pattern: None,
            skip_tags: None,
            ignore_tags: None,
            date_order: None,
            sort_commits: None,
            link_parsers: Some(vec![
                LinkParser {
                    pattern: Regex::new("#(\\d+)").unwrap(),
                    href: String::from("https://github.com/$1"),
                    text: None,
                },
                LinkParser {
                    pattern: Regex::new("https://github.com/(.*)").unwrap(),
                    href: String::from("https://github.com/$1"),
                    text: Some(String::from("$1")),
                },
            ]),
        };
        let release_date = self.release_timestamp();
        let commits = self
            .commits
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
        }
    }

    fn release_timestamp(&self) -> i64 {
        let release_date = self.release_date.unwrap_or_else(|| Utc::now().date());
        let difference = release_date - Utc.ymd(1970, 1, 1);
        difference.num_seconds()
    }
}

pub struct Changelog<'a> {
    release: Release<'a>,
}

impl<'a> Changelog<'a> {
    pub fn full(&self) -> String {
        format!("{CHANGELOG_HEADER}\n{}", self.body())
    }

    fn body(&self) -> String {
        let changelog_config = changelog_config();
        let template = Template::new(changelog_config.body.unwrap()).unwrap();
        template.render(&self.release).unwrap()
    }

    pub fn update(&self, old_changelog: &str) -> String {
        let separator = "## [Unreleased]";
        let unreleased_idx = old_changelog.find(separator);
        let mut new_changelog = old_changelog.to_string();
        let update_idx = unreleased_idx
            .map(|idx| {
                let mut idx = idx + separator.len();
                add_new_line_if_not_present(&mut new_changelog, idx);
                idx += 1;
                add_new_line_if_not_present(&mut new_changelog, idx);
                idx + 1
            })
            .unwrap_or(0);

        let body = format!("{}\n", &self.body());
        new_changelog.insert_str(update_idx, &body);
        new_changelog
    }
}

fn add_new_line_if_not_present(text: &mut String, idx: usize) {
    if let Some(c) = text.chars().nth(idx) {
        if c == '\n' {
            return;
        }
    }
    text.insert(idx, '\n');
}

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
fn changelog_config() -> ChangelogConfig {
    ChangelogConfig {
        header: Some(String::from("this is a changelog")),
        body: Some(String::from(
            r#"## [{{ version | trim_start_matches(pat="v") }}] - {{ timestamp | date(format="%Y-%m-%d") }}
{% for group, commits in commits | group_by(attribute="group") %}
### {{ group | upper_first }}
{% for commit in commits %}
{%- if commit.scope -%}
- *({{commit.scope}})* {{ commit.message }}{%- if commit.links %} ({% for link in commit.links %}[{{link.text}}]({{link.href}}) {% endfor -%}){% endif %}
{% else -%}
- {{ commit.message }}
{% endif -%}
{% if commit.breaking -%}
{% raw %}  {% endraw %}- **BREAKING**: {{commit.breaking_description}}
{% endif -%}
{% endfor -%}
{% endfor %}"#,
        )),
        footer: Some(String::from("eoc - end of changelog")),
        trim: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn changelog_body_is_generated() {
        let commits = vec!["fix: myfix", "simple update"];
        let changelog = ChangelogBuilder::new(commits, "1.1.1")
            .with_release_date(Utc.ymd(2015, 5, 15))
            .build();
        expect_test::expect![[r####"
            ## [1.1.1] - 2015-05-15

            ### Fixed
            - myfix

            ### Other
            - simple update
        "####]]
        .assert_eq(&changelog.body());
    }

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
        .assert_eq(&changelog.full());
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
        let new = changelog.update(&old);
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
        let new = changelog.update(old);
        expect_test::expect![[r####"
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
