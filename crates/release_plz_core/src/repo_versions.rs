use anyhow::anyhow;
use cargo_metadata::semver::Version;
use git_cmd::Repo;
use regex::Regex;

use std::collections::BTreeMap;

lazy_static::lazy_static! {
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
/// * `RepoVersions::Single` if all version tags are of the form 'v{version}' and
///   `contains_multiple_pub_packages` is `false`
/// * `RepoVersions::ByPackage` if all version tags are of the form '{package}-v{version}' and
///   `contains_multiple_pub_packages` is `true`
/// * Panics otherwise.
pub fn get_repo_versions(
    repo: &Repo,
    contains_multiple_pub_packages: bool,
) -> anyhow::Result<Option<RepoVersions>> {
    let Some(tags) = repo.get_tags_version_sorted(true) else {
        return Err(anyhow!("cannot get tags version sorted"));
    };

    let (with_package, no_package): (Vec<_>, Vec<_>) = tags
        .iter()
        .filter_map(|tag| VERSION_TAG_RE.captures(tag))
        .partition(|caps| caps.get(1).is_some());

    if with_package.is_empty() && no_package.is_empty() {
        Ok(None)
    } else if !with_package.is_empty() && !no_package.is_empty() {
        return Err(anyhow!(
            "Found version tags with and without a package name in repo at: {:?}",
            repo.directory()
        ));
    } else if !with_package.is_empty() && contains_multiple_pub_packages {
        let versions_map: BTreeMap<String, Version> =
            with_package.iter().fold(BTreeMap::new(), |mut map, caps| {
                let package = caps.get(1).unwrap().as_str();
                // The tag list is version-sorted in reverse, so the first tag we encounter
                // for each package is the most recent version.
                if !map.contains_key(package) {
                    if let Ok(version) = Version::parse(caps.get(2).unwrap().as_str()) {
                        map.insert(package.to_string(), version);
                    }
                }
                map
            });

        return Ok(Some(RepoVersions::ByPackage(versions_map)));
    } else if !no_package.is_empty() && !contains_multiple_pub_packages {
        return Ok(Version::parse(
            no_package
                .first()
                .expect("no_package is not empty")
                .get(2)
                .expect("regex capture was empty for index 2")
                .as_str(),
        )
        .map(RepoVersions::Single)
        .ok());
    } else {
        return Err(anyhow!(
            "Mismatch between version tag format and `contains_multiple_pub_packages`"
        ));
    }
}
