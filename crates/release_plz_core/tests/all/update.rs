use std::fs;

use cargo_metadata::Version;
use fs_extra::dir;
use git_cmd::git_in_dir;
use release_plz_core::{are_packages_equal, read_package, UpdateRequest, CARGO_TOML};
use tempfile::tempdir;

use crate::init_project;

#[test]
fn up_to_date_project_is_not_touched() {
    test_logs::init();
    let local_project_dir = tempdir().unwrap();
    let local_project = local_project_dir.as_ref().join("myproject");
    init_project(&local_project);

    let remote_project = tempdir().unwrap();
    dir::copy(
        &local_project,
        remote_project.as_ref(),
        &dir::CopyOptions::default(),
    )
    .unwrap();

    let update_request = UpdateRequest::new(local_project.join(CARGO_TOML))
        .unwrap()
        .with_remote_manifest(remote_project.as_ref().join("myproject").join(CARGO_TOML))
        .unwrap();
    release_plz_core::update(&update_request).unwrap();

    // the update should have not changed anything
    assert!(are_packages_equal(
        &local_project,
        &remote_project.as_ref().join("myproject")
    ));
}

#[test]
fn version_is_updated_when_project_changed() {
    test_logs::init();
    let local_project_dir = tempdir().unwrap();
    let local_project = local_project_dir.as_ref().join("myproject");
    init_project(&local_project);

    let remote_project = tempdir().unwrap();
    dir::copy(
        &local_project,
        remote_project.as_ref(),
        &dir::CopyOptions::default(),
    )
    .unwrap();

    fs::write(
        local_project.join("src").join("lib.rs"),
        "do amazing things",
    )
    .unwrap();
    git_in_dir(&local_project, &["add", "."]).unwrap();
    git_in_dir(&local_project, &["commit", "-m", "feat: do awesome stuff"]).unwrap();

    let update_request = UpdateRequest::new(local_project.join(CARGO_TOML))
        .unwrap()
        .with_remote_manifest(remote_project.as_ref().join("myproject").join(CARGO_TOML))
        .unwrap();
    release_plz_core::update(&update_request).unwrap();

    // the update should have changed the version
    assert!(!are_packages_equal(
        &local_project,
        &remote_project.as_ref().join("myproject")
    ));

    let local_package = read_package(local_project).unwrap();
    assert_eq!(local_package.version, Version::new(0, 1, 1));
}
