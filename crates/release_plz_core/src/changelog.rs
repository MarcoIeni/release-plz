use git_cliff_core::{
    commit::Commit,
    config::{ChangelogConfig, CommitParser, GitConfig, LinkParser},
    regex::Regex,
    release::Release,
    template::Template,
};

struct Changelog<'a> {
    release: Release<'a>,
}
impl<'a> Changelog<'a> {
    fn new<I: Into<String>>(commits: Vec<I>) -> Self {
        let git_config = GitConfig {
            conventional_commits: Some(true),
            filter_unconventional: Some(false),
            commit_parsers: Some(vec![
                CommitParser {
                    message: Regex::new("^feat").ok(),
                    body: None,
                    group: Some(String::from("shiny features")),
                    default_scope: None,
                    skip: None,
                },
                CommitParser {
                    message: Regex::new("^fix").ok(),
                    body: None,
                    group: Some(String::from("fix bugs")),
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
            ]),
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
        let commits = commits
            .into_iter()
            .map(|c| Commit {
                id: "commit_id".to_string(),
                message: c.into(),
                conv: None,
                group: None,
                scope: None,
                links: vec![],
            })
            .filter_map(|c| c.process(&git_config).ok())
            .collect();

        Self {
            release: Release {
                version: Some("1.1.1".to_string()),
                commits,
                commit_id: Some("dsaujkldjksa".to_string()),
                timestamp: 1111,
                previous: None,
            },
        }
    }

    fn full(&self) -> String {
        let body = self.body();
        let header = r#"# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]"#;

        format!("{header}\n{body}")
    }

    fn changelog_config() -> ChangelogConfig {
        ChangelogConfig {
            header: Some(String::from("this is a changelog")),
            body: Some(String::from(
                r#"
## Release {{ version }}
{% for group, commits in commits | group_by(attribute="group") %}
### {{ group }}
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

    fn body(&self) -> String {
        let changelog_config = Self::changelog_config();
        let template = Template::new(changelog_config.body.unwrap()).unwrap();
        template.render(&self.release).unwrap()
    }

    fn update(&self, old_changelog: &str) -> String {
        let separator = "## [Unreleased]";
        let idx = old_changelog.find(separator).unwrap();
        let mut new_changelog = old_changelog.to_string();
        let new_release = format!("\n{}", &self.body());
        new_changelog.insert_str(idx + separator.len(), &new_release);
        new_changelog
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn changelog_entries_are_generated() {
        let commits = vec!["fix: myfix", "simple update"];
        let changelog = Changelog::new(commits);
        println!("{}", changelog.full());
    }

    #[test]
    fn changelog_id_updated() {
        let commits = vec!["fix: myfix", "simple update"];
        let changelog = Changelog::new(commits);
        let old = r#"# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## Release 1.1.0

### fix bugs
- my awesomefix

### other
- complex update"#;

        println!("{}", changelog.update(old));
    }
}
