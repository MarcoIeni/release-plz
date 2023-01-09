use next_version::NextVersion;
use semver::Version;

#[test]
fn commit_without_semver_prefix_increments_pre_release_version() {
    let commits = vec!["my change"];
    let version = Version::parse("1.0.0-alpha.2").unwrap();
    let expected = Version::parse("1.0.0-alpha.3").unwrap();
    assert_eq!(version.next(commits), expected);
}
