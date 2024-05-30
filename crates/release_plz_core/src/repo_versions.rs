use git_cmd::Repo;
use regex::Regex;

/// Returns the latest tag of the repository that matches the regex as a String.
/// * `None` if there are no version tags found matching the regex,
/// * `Tag` if at least one tag is matching the regex
pub fn get_repo_versions(repo: &Repo) -> Option<String> {
    /*
    Regex explanation:
    - \b asserts a word boundary to ensure the match is not part of a longer word.
    - ([a-zA-Z0-9_-]+-)? optionally matches a package name consisting of alphanumeric characters, underscores, or hyphens followed by a hyphen. The ? makes this group optional.
    - v matches the letter 'v'.
    - \d+\.\d+\.\d+ matches the version number in x.x.x format, where \d+ matches one or more digits and \. matches a literal period.
    - \b asserts another word boundary to ensure the match is not part of a longer string.

    Examples:
    v1.2.3 matches.
    v0.2.3 matches.
    tokio-v1.2.3 matches.
    parser-v0.1.2 matches.
    */
    let regex = Regex::new(r"\b([a-zA-Z0-9_-]+-)?(v\d+\.\d+\.\d+)\b").unwrap();

    let Some(tags) = repo.get_tags_version_sorted(true) else {
        return None;
    };

    let matching_tags = tags
        .iter()
        .filter_map(|tag| regex.captures(tag))
        .collect::<Vec<_>>();

    if matching_tags.is_empty() {
        None
    } else {
        return Some(
            matching_tags
                .first()
                .expect("we ensured there is at least one matching tag")
                .iter()
                .last()
                .expect("last item should be present")
                .expect("regex capture cannot be empty")
                .as_str()
                .to_owned(),
        );
    }
}
