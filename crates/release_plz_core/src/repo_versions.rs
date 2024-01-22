use cargo_metadata::semver::Version;
use git_cmd::Repo;
use regex::Regex;
use std::collections::BTreeMap;

lazy_static::lazy_static! {
    // match PR/issue numbers, e.g. `#123`
    static ref VERSION_TAG_RE: Regex = Regex::new("(?:(.+)-)?v(.+)").unwrap();
}

pub enum RepoVersions {
    /// Most recent version for the repository's single public package.
    Single(Version),
    /// Most recent version for each of the repository's public packages
    ByPackage(BTreeMap<String, Version>),
}

impl RepoVersions {
    pub fn get_package_version(&self, package_name: &str) -> Option<&Version> {
        match self {
            Self::Single(version) => Some(version),
            Self::ByPackage(versions) => versions.get(package_name),
        }
    }
}

/// Gets sorted tags from `Repo` and returns:
/// * `None` if there are no version tags
/// * `RepoVersions::Single` if all verison tags are of the form 'v{version}' and
///   `contains_multiple_pub_packages` is `false`
/// * `RepoVersions::ByPackage` if all verison tags are of the form '{package}-v{version}' and
///   `contains_multiple_pub_packages` is `true`
/// * Panics otherwise.
pub fn get_repo_versions(
    repo: &Repo,
    contains_multiple_pub_packages: bool,
) -> Option<RepoVersions> {
    repo.get_tags_version_sorted(true)
        .map(|tags| {
            assert!(!tags.is_empty());
            let (with_package, no_package): (Vec<_>, Vec<_>) = tags
                .iter()
                .filter_map(|tag| VERSION_TAG_RE.captures(tag))
                .partition(|caps| caps.get(1).is_some());
            if with_package.is_empty() && no_package.is_empty() {
                None
            } else if !(with_package.is_empty() || no_package.is_empty()) {
                panic!(
                    "Found version tags with and without a package name in repo at: {:?}",
                    repo.directory()
                );
            } else if !with_package.is_empty() && contains_multiple_pub_packages {
                Some(RepoVersions::ByPackage(with_package.iter().fold(
                    BTreeMap::new(),
                    |mut map, caps| {
                        let package = caps.get(1).unwrap().as_str();
                        // The tag list is version-sorted in reverse, so the first tag we encounter
                        // for each package is the most recent version.
                        if !map.contains_key(package) {
                            if let Ok(version) = Version::parse(caps.get(2).unwrap().as_str()) {
                                map.insert(package.to_string(), version);
                            }
                        }
                        map
                    },
                )))
            } else if !no_package.is_empty() && !contains_multiple_pub_packages {
                Version::parse(no_package.first().unwrap().get(2).unwrap().as_str())
                    .map(|version| RepoVersions::Single(version))
                    .ok()
            } else {
                panic!("Mismatch between version tag format and `contains_multiple_pub_packages`");
            }
        })
        .flatten()
}
