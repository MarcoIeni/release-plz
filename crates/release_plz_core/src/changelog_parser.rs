use std::{fs::read_to_string, path::Path};

use anyhow::Context;

pub fn last_changes(changelog: &Path) -> anyhow::Result<Option<String>> {
    let changelog = read_to_string(changelog).context("can't read changelog file")?;
    let parser = ChangelogParser::new(&changelog)?;
    let last_release = parser.last_release().map(|r| r.notes.to_string());
    Ok(last_release)
}

pub fn last_version_from_str(changelog: &str) -> anyhow::Result<Option<String>> {
    let parser = ChangelogParser::new(changelog)?;
    let last_release = parser.last_release().map(|r| r.version.to_string());
    Ok(last_release)
}

pub struct ChangelogParser<'a> {
    changelog: parse_changelog::Changelog<'a>,
}

impl<'a> ChangelogParser<'a> {
    pub fn new(changelog_text: &'a str) -> anyhow::Result<Self> {
        let changelog = parse_changelog::parse(changelog_text).context("can't parse changelog")?;
        Ok(Self { changelog })
    }

    fn last_release(&self) -> Option<&parse_changelog::Release> {
        let last_release = release_at(&self.changelog, 0)?;
        let last_release = if last_release.version.to_lowercase().contains("unreleased") {
            release_at(&self.changelog, 1)?
        } else {
            last_release
        };
        Some(last_release)
    }
}

fn release_at<'a>(
    changelog: &'a parse_changelog::Changelog,
    index: usize,
) -> Option<&'a parse_changelog::Release<'a>> {
    let release = changelog.get_index(index)?.1;
    Some(release)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn last_changes_from_str(changelog: &str) -> String {
        let parser = ChangelogParser::new(changelog).unwrap();
        parser.last_release().unwrap().notes.to_string()
    }

    #[test]
    fn changelog_with_unreleased_section_is_parsed() {
        let changelog = "\
# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.5] - 2022-12-16

### Added
- Add function to retrieve default branch (#372)

## [0.2.4] - 2022-12-12

### Changed
- improved error message
";
        let changes = last_changes_from_str(changelog);
        let expected_changes = "\
### Added
- Add function to retrieve default branch (#372)";
        assert_eq!(changes, expected_changes);
    }

    #[test]
    fn changelog_without_unreleased_section_is_parsed() {
        let changelog = "\
# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.5](https://github.com/MarcoIeni/release-plz/compare/git_cmd-v0.2.4...git_cmd-v0.2.5) - 2022-12-16

### Added
- Add function to retrieve default branch (#372)

## [0.2.4] - 2022-12-12

### Changed
- improved error message
";
        let changes = last_changes_from_str(changelog);
        let expected_changes = "\
### Added
- Add function to retrieve default branch (#372)";
        assert_eq!(changes, expected_changes);
    }
}
