use std::{fs::read_to_string, path::Path};

use anyhow::Context;

pub fn last_changes(changelog: &Path) -> anyhow::Result<String> {
    let changelog = read_to_string(changelog).context("can't read changelog file")?;
    last_changes_from_str(&changelog)
}

fn last_changes_from_str(changelog: &str) -> anyhow::Result<String> {
    let changelog = parse_changelog::parse(changelog).context("can't parse changelog")?;
    let last_release = release_at(&changelog, 0);
    let last_changes = if last_release.version.to_lowercase().contains("unreleased") {
        release_at(&changelog, 1).notes
    } else {
        last_release.notes
    };
    Ok(last_changes.to_string())
}

fn release_at<'a>(
    changelog: &'a parse_changelog::Changelog,
    index: usize,
) -> &'a parse_changelog::Release<'a> {
    changelog
        .get_index(index)
        .expect("can't find latest release in changelog")
        .1
}

#[cfg(test)]
mod tests {
    use super::last_changes_from_str;

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
        let changes = last_changes_from_str(changelog).unwrap();
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
        let changes = last_changes_from_str(changelog).unwrap();
        let expected_changes = "\
### Added
- Add function to retrieve default branch (#372)";
        assert_eq!(changes, expected_changes);
    }
}
