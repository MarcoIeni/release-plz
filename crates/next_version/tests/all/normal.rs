use next_version::{NextVersion, VersionUpdater};
use semver::Version;

#[test]
fn commit_without_semver_prefix_increments_patch_version() {
    let commits = ["my change"];
    let version = Version::new(1, 2, 3);
    assert_eq!(version.next(commits), Version::new(1, 2, 4));
}

#[test]
fn commit_with_fix_semver_prefix_increments_patch_version() {
    let commits = ["my change", "fix: serious bug"];
    let version = Version::new(1, 2, 3);
    assert_eq!(version.next(commits), Version::new(1, 2, 4));
}

#[test]
fn commit_with_feat_semver_prefix_increments_patch_version() {
    let commits = ["feat: make coffe"];
    let version = Version::new(1, 3, 3);
    assert_eq!(version.next(commits), Version::new(1, 4, 0));
}

#[test]
fn commit_with_feat_semver_prefix_increments_patch_version_when_major_is_zero() {
    let commits = ["feat: make coffee"];
    let version = Version::new(0, 2, 3);
    assert_eq!(version.next(commits), Version::new(0, 2, 4));
}

#[test]
fn commit_with_feat_semver_prefix_increments_minor_version_when_major_is_zero() {
    let commits = ["feat: make coffee"];
    let version = Version::new(0, 2, 3);
    assert_eq!(
        VersionUpdater::new()
            .with_features_always_increment_minor(true)
            .with_breaking_always_increment_major(false)
            .increment(&version, commits),
        Version::new(0, 3, 0)
    );
}

#[test]
fn commit_with_breaking_change_increments_major_version() {
    let commits = ["feat!: break user"];
    let version = Version::new(1, 2, 3);
    assert_eq!(version.next(commits), Version::new(2, 0, 0));
}

#[test]
fn commit_with_breaking_change_increments_minor_version_when_major_is_zero() {
    let commits = ["feat!: break user"];
    let version = Version::new(0, 2, 3);
    assert_eq!(version.next(commits), Version::new(0, 3, 0));
}

#[test]
fn commit_with_breaking_change_increments_major_version_when_major_is_zero() {
    let commits = ["feat!: break user"];
    let version = Version::new(0, 2, 3);
    assert_eq!(
        VersionUpdater::new()
            .with_features_always_increment_minor(false)
            .with_breaking_always_increment_major(true)
            .increment(&version, commits),
        Version::new(1, 0, 0)
    );
}
