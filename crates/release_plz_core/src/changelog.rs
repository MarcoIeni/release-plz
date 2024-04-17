use anyhow::Context;
use chrono::{NaiveDate, TimeZone, Utc};
use git_cliff_core::{
    changelog::Changelog as GitCliffChangelog,
    commit::Commit,
    config::{Bump, ChangelogConfig, CommitParser, Config, GitConfig, RemoteConfig},
    release::Release,
};
use regex::Regex;
use tracing::warn;

use crate::{changelog_header::replace_unreleased, changelog_parser};

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
        let config = self.changelog_config(None, self.release_link.as_deref());
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
        if is_version_unchanged(&self.release) {
            tracing::debug!("Version is unchanged, not updating changelog");
            // The changelog already contains this version, so we don't update the changelog.
            return Ok(old_changelog);
        }
        let old_header = changelog_parser::parse_header(&old_changelog);
        let config = self.changelog_config(old_header, self.release_link.as_deref());
        let changelog = GitCliffChangelog::new(vec![self.release], &config)
            .context("error while building changelog")?;
        let mut out = Vec::new();
        tracing::debug!("updating changelog: {:#?}", changelog);
        changelog
            .prepend(old_changelog, &mut out)
            .context("cannot update changelog")?;
        String::from_utf8(out).context("cannot convert bytes to string")
    }

    fn changelog_config(&self, header: Option<String>, release_link: Option<&str>) -> Config {
        let user_config = self.config.clone().unwrap_or(default_git_cliff_config());
        Config {
            changelog: apply_defaults_to_changelog_config(
                user_config.changelog,
                header,
                release_link,
            ),
            git: apply_defaults_to_git_config(user_config.git),
            remote: user_config.remote,
            bump: Bump::default(),
        }
    }
}

/// Apply release-plz defaults
fn apply_defaults_to_changelog_config(
    changelog: ChangelogConfig,
    header: Option<String>,
    release_link: Option<&str>,
) -> ChangelogConfig {
    let default_changelog_config = default_changelog_config(header, release_link);

    ChangelogConfig {
        header: changelog.header.or(default_changelog_config.header),
        body: changelog.body.or(default_changelog_config.body),
        trim: changelog.trim.or(default_changelog_config.trim),
        ..changelog
    }
}

/// Apply release-plz defaults
fn apply_defaults_to_git_config(git: GitConfig) -> GitConfig {
    let default_git_config = default_git_config();

    GitConfig {
        conventional_commits: git
            .conventional_commits
            .or(default_git_config.conventional_commits),
        filter_unconventional: git
            .filter_unconventional
            .or(default_git_config.filter_unconventional),
        commit_parsers: git.commit_parsers.or(default_git_config.commit_parsers),
        filter_commits: git.filter_commits.or(default_git_config.filter_commits),
        sort_commits: git.sort_commits.or(default_git_config.sort_commits),
        ..git
    }
}

fn is_version_unchanged(release: &Release) -> bool {
    let previous_version = release.previous.as_ref().and_then(|r| r.version.as_deref());
    let new_version = release.version.as_deref();
    previous_version == new_version
}

fn default_git_cliff_config() -> Config {
    Config {
        changelog: ChangelogConfig::default(),
        git: GitConfig::default(),
        remote: RemoteConfig::default(),
        bump: Bump::default(),
    }
}

pub struct ChangelogBuilder<'a> {
    commits: Vec<Commit<'a>>,
    version: String,
    previous_version: Option<String>,
    config: Option<Config>,
    release_date: Option<NaiveDate>,
    release_link: Option<String>,
}

impl<'a> ChangelogBuilder<'a> {
    pub fn new(commits: Vec<Commit<'a>>, version: impl Into<String>) -> Self {
        Self {
            commits,
            version: version.into(),
            previous_version: None,
            config: None,
            release_date: None,
            release_link: None,
        }
    }

    pub fn with_previous_version(self, previous_version: impl Into<String>) -> Self {
        Self {
            previous_version: Some(previous_version.into()),
            ..self
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

    pub fn build(self) -> Changelog<'a> {
        let mut git_config = self
            .config
            .clone()
            .map(|c| c.git)
            .unwrap_or_else(default_git_config);
        git_config = apply_defaults_to_git_config(git_config);
        let release_date = self.release_timestamp();
        let mut commits: Vec<_> = self
            .commits
            .into_iter()
            .filter_map(|c| c.process(&git_config).ok())
            .collect();

        match git_config.sort_commits.map(|s| s.to_lowercase()).as_deref() {
            Some("oldest") => {
                commits.reverse();
            }
            Some("newest") | None => {
                // commits are already sorted from newest to oldest, we don't need to do anything
            }
            Some(other) => {
                warn!("Invalid setting for sort_commits: '{other}'. Valid values are 'newest' and 'oldest'.")
            }
        }

        let previous = self.previous_version.map(|ver| Release {
            version: Some(ver),
            commits: vec![],
            commit_id: None,
            timestamp: 0,
            previous: None,
        });

        Changelog {
            release: Release {
                version: Some(self.version),
                commits,
                commit_id: None,
                timestamp: release_date,
                previous: previous.map(Box::new),
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
            .map(|d| Utc.from_utc_datetime(&d))
            .unwrap_or_else(Utc::now)
            .timestamp()
    }
}

fn default_git_config() -> GitConfig {
    GitConfig {
        conventional_commits: Some(true),
        filter_unconventional: Some(false),
        commit_parsers: Some(kac_commit_parsers()),
        filter_commits: Some(true),
        tag_pattern: None,
        skip_tags: None,
        split_commits: None,
        protect_breaking_commits: None,
        topo_order: None,
        ignore_tags: None,
        limit_commits: None,
        sort_commits: Some("newest".to_string()),
        commit_preprocessors: None,
        link_parsers: None,
    }
}

fn commit_parser(regex: &str, group: &str) -> CommitParser {
    CommitParser {
        message: Regex::new(regex).ok(),
        body: None,
        group: Some(group.to_string()),
        default_scope: None,
        scope: None,
        skip: None,
        field: None,
        pattern: None,
        sha: None,
    }
}

/// Commit parsers based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
fn kac_commit_parsers() -> Vec<CommitParser> {
    vec![
        commit_parser("^feat", "added"),
        commit_parser("^changed", "changed"),
        commit_parser("^deprecated", "deprecated"),
        commit_parser("^removed", "removed"),
        commit_parser("^fix", "fixed"),
        commit_parser("^security", "security"),
        commit_parser(".*", "other"),
    ]
}

fn default_changelog_config(header: Option<String>, release_link: Option<&str>) -> ChangelogConfig {
    let header = header.unwrap_or(String::from(CHANGELOG_HEADER));
    let header = replace_unreleased(&header, "aaaa");
    ChangelogConfig {
        header: Some(header),
        body: Some(default_changelog_body_config(release_link)),
        footer: None,
        postprocessors: None,
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
    use crate::NO_COMMIT_ID;

    use super::*;

    #[test]
    fn changelog_entries_are_generated() {
        let commits = vec![
            Commit::new(NO_COMMIT_ID.to_string(), "fix: myfix".to_string()),
            Commit::new(NO_COMMIT_ID.to_string(), "simple update".to_string()),
        ];
        let changelog = ChangelogBuilder::new(commits, "1.1.1")
            .with_release_date(NaiveDate::from_ymd_opt(2015, 5, 15).unwrap())
            .build();

        expect_test::expect![[r#"
            # Changelog
            All notable changes to this project will be documented in this file.

            The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
            and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

            ## [Unreleased](aaaa)

            ## [1.1.1] - 2015-05-15

            ### Fixed
            - myfix

            ### Other
            - simple update
        "#]]
        .assert_eq(&changelog.generate());
    }

    #[test]
    fn changelog_entry_with_link_is_generated() {
        let commits = vec![Commit::new(
            NO_COMMIT_ID.to_string(),
            "fix: myfix".to_string(),
        )];
        let changelog = ChangelogBuilder::new(commits, "1.1.1")
            .with_release_date(NaiveDate::from_ymd_opt(2015, 5, 15).unwrap())
            .with_release_link("https://github.com/MarcoIeni/release-plz/compare/release-plz-v0.2.24...release-plz-v0.2.25")
            .build();

        expect_test::expect![[r#"
            # Changelog
            All notable changes to this project will be documented in this file.

            The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
            and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

            ## [Unreleased](aaaa)

            ## [1.1.1](https://github.com/MarcoIeni/release-plz/compare/release-plz-v0.2.24...release-plz-v0.2.25) - 2015-05-15

            ### Fixed
            - myfix
        "#]]
        .assert_eq(&changelog.generate());
    }

    #[test]
    fn generated_changelog_is_updated_correctly() {
        let commits = vec![
            Commit::new(NO_COMMIT_ID.to_string(), "fix: myfix".to_string()),
            Commit::new(NO_COMMIT_ID.to_string(), "simple update".to_string()),
        ];
        let changelog = ChangelogBuilder::new(commits, "1.1.1")
            .with_release_date(NaiveDate::from_ymd_opt(2015, 5, 15).unwrap())
            .build();

        let generated_changelog = changelog.generate();

        let commits = vec![
            Commit::new(NO_COMMIT_ID.to_string(), "fix: myfix2".to_string()),
            Commit::new(NO_COMMIT_ID.to_string(), "complex update".to_string()),
        ];
        let changelog = ChangelogBuilder::new(commits, "1.1.2")
            .with_release_date(NaiveDate::from_ymd_opt(2015, 5, 15).unwrap())
            .build();

        expect_test::expect![[r#"
            # Changelog
            All notable changes to this project will be documented in this file.

            The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
            and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

            ## [Unreleased](aaaa)

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
        "#]]
        .assert_eq(&changelog.prepend(generated_changelog).unwrap());
    }

    #[test]
    fn changelog_is_updated() {
        let commits = vec![
            Commit::new(NO_COMMIT_ID.to_string(), "fix: myfix".to_string()),
            Commit::new(NO_COMMIT_ID.to_string(), "simple update".to_string()),
        ];
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
        expect_test::expect![[r#"
            # Changelog
            All notable changes to this project will be documented in this file.

            The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
            and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

            ## [Unreleased](aaaa)

            ## [1.1.1] - 2015-05-15

            ### Fixed
            - myfix

            ### Other
            - simple update
            # Changelog
            All notable changes to this project will be documented in this file.

            The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
            and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

            ## [Unreleased]

            ## [1.1.0] - 1970-01-01

            ### fix bugs
            - my awesomefix

            ### other
            - complex update
        "#]]
        .assert_eq(&new);
    }

    #[test]
    fn changelog_without_header_is_updated() {
        let commits = vec![
            Commit::new(NO_COMMIT_ID.to_string(), "fix: myfix".to_string()),
            Commit::new(NO_COMMIT_ID.to_string(), "simple update".to_string()),
        ];
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
        expect_test::expect![[r#"
            # Changelog
            All notable changes to this project will be documented in this file.

            The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
            and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

            ## [Unreleased](aaaa)

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
        "#]]
        .assert_eq(&new.unwrap());
    }

    #[test]
    fn changelog_has_commit_id() {
        let commits = vec![
            Commit::new("1111111".to_string(), "fix: myfix".to_string()),
            Commit::new(
                NO_COMMIT_ID.to_string(),
                "chore: something else".to_string(),
            ),
        ];
        let changelog = ChangelogBuilder::new(commits, "1.1.1")
            .with_release_date(NaiveDate::from_ymd_opt(2015, 5, 15).unwrap())
            .with_config(Config {
                changelog: ChangelogConfig {
                    header: Some("".to_string()),
                    body: Some(
                        r"{%- for commit in commits %}
                            {{ commit.message }} - {{ commit.id }}
                        {% endfor -%}"
                            .to_string(),
                    ),
                    ..ChangelogConfig::default()
                },
                git: GitConfig::default(),
                remote: RemoteConfig::default(),
                bump: Bump::default(),
            })
            .build();

        expect_test::expect![[r####"

            myfix - 1111111

            something else - 0000000
        "####]]
        .assert_eq(&changelog.generate());
    }

    #[test]
    fn changelog_sort_newest() {
        let commits = vec![
            Commit::new("1111111".to_string(), "fix: myfix".to_string()),
            Commit::new("0000000".to_string(), "fix: another fix".to_string()),
        ];
        let changelog = ChangelogBuilder::new(commits, "1.1.1")
            .with_release_date(NaiveDate::from_ymd_opt(2015, 5, 15).unwrap())
            .with_config(Config {
                changelog: default_changelog_config(None, None),
                git: GitConfig {
                    sort_commits: Some("oldest".to_string()),
                    ..GitConfig::default()
                },
                remote: RemoteConfig::default(),
                bump: Bump::default(),
            })
            .build();

        expect_test::expect![[r#"
            # Changelog
            All notable changes to this project will be documented in this file.

            The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
            and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

            ## [Unreleased](aaaa)

            ## [1.1.1] - 2015-05-15

            ### Fixed
            - another fix
            - myfix
        "#]]
        .assert_eq(&changelog.generate());
    }
}

#[test]
fn empty_changelog_is_updated() {
    test_logs::init();
    let commits = vec![
        Commit::new(crate::NO_COMMIT_ID.to_string(), "fix: myfix".to_string()),
        Commit::new(crate::NO_COMMIT_ID.to_string(), "simple update".to_string()),
    ];
    let changelog = ChangelogBuilder::new(commits, "1.1.1")
        .with_release_date(NaiveDate::from_ymd_opt(2015, 5, 15).unwrap())
        .build();
    let new = changelog.prepend(CHANGELOG_HEADER).unwrap();
    expect_test::expect![[r#"
        # Changelog
        All notable changes to this project will be documented in this file.

        The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
        and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

        ## [Unreleased](aaaa)

        ## [1.1.1] - 2015-05-15

        ### Fixed
        - myfix

        ### Other
        - simple update
    "#]]
    .assert_eq(&new);
}
