use next_version::{NextVersion, NextVersionConfig};
use semver::Version;

#[test]
fn commit_without_semver_prefix_increments_patch_version() {
    let commits = vec!["my change"];
    let version = Version::new(1, 2, 3);
    assert_eq!(version.next(commits, None), Version::new(1, 2, 4));
}

#[test]
fn commit_with_fix_semver_prefix_increments_patch_version() {
    let commits = vec!["my change", "fix: serious bug"];
    let version = Version::new(1, 2, 3);
    assert_eq!(version.next(commits, None), Version::new(1, 2, 4));
}

#[test]
fn commit_with_feat_semver_prefix_increments_patch_version() {
    let commits = vec!["feat: make coffe"];
    let version = Version::new(1, 3, 3);
    assert_eq!(version.next(commits, None), Version::new(1, 4, 0));
}

#[test]
fn commit_with_feat_semver_prefix_increments_patch_version_when_major_is_zero() {
    let commits = vec!["feat: make coffee"];
    let version = Version::new(0, 2, 3);
    assert_eq!(version.next(commits, None), Version::new(0, 2, 4));
}

#[test]
fn commit_with_feat_semver_prefix_increments_minor_version_when_major_is_zero() {
    let commits = vec!["feat: make coffee"];
    let version = Version::new(0, 2, 3);
    assert_eq!(
        version.next(
            commits,
            Some(NextVersionConfig {
                uncontrolled_minor_bump: true,
                initial_major_increment: false
            })
        ),
        Version::new(0, 3, 0)
    );
}

#[test]
fn commit_with_breaking_change_increments_major_version() {
    let commits = vec!["feat!: break user"];
    let version = Version::new(1, 2, 3);
    assert_eq!(version.next(commits, None), Version::new(2, 0, 0));
}

#[test]
fn commit_with_breaking_change_increments_minor_version_when_major_is_zero() {
    let commits = vec!["feat!: break user"];
    let version = Version::new(0, 2, 3);
    assert_eq!(version.next(commits, None), Version::new(0, 3, 0));
}

#[test]
fn commit_with_breaking_change_increments_major_version_when_major_is_zero() {
    let commits = vec!["feat!: break user"];
    let version = Version::new(0, 2, 3);
    assert_eq!(
        version.next(
            commits,
            Some(NextVersionConfig {
                uncontrolled_minor_bump: false,
                initial_major_increment: true
            })
        ),
        Version::new(1, 0, 0)
    );
}
