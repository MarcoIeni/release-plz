use std::{
    fs, io,
    path::{Path, PathBuf},
    process::{Command, Output},
};

use folder_compare::FolderCompare;
use fs_extra::dir;
use release_plz_core::UpdateRequest;
use tempfile::tempdir;

fn join_cargo_toml(project: &Path) -> PathBuf {
    project.join("Cargo.toml")
}

fn git_in_dir(dir: &Path, args: &[&str]) -> io::Result<Output> {
    let args: Vec<&str> = args.iter().map(|s| s.trim()).collect();
    Command::new("git").arg("-C").arg(dir).args(args).output()
}

fn init_project(project: &Path) {
    Command::new("cargo")
        .arg("new")
        .arg(project)
        .output()
        .unwrap();

    git_in_dir(project, &["init"]).unwrap();
    fs::write(project.join("README.md"), "# my awesome project").unwrap();
    git_in_dir(project, &["add", "."]).unwrap();
    git_in_dir(project, &["commit", "-m", "add README"]).unwrap();
}

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

    let update_request = UpdateRequest::new(join_cargo_toml(&local_project))
        .unwrap()
        .with_remote_manifest(join_cargo_toml(&remote_project.as_ref().join("myproject")))
        .unwrap();
    release_plz_core::update(&update_request).unwrap();

    // the update should have not changed anything
    assert!(are_dir_equal(
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

    let update_request = UpdateRequest::new(join_cargo_toml(&local_project))
        .unwrap()
        .with_remote_manifest(join_cargo_toml(&remote_project.as_ref().join("myproject")))
        .unwrap();
    release_plz_core::update(&update_request).unwrap();

    // the update should have changed the version
    assert!(!are_dir_equal(
        &local_project,
        &remote_project.as_ref().join("myproject")
    ));
}

fn are_dir_equal(first: &Path, second: &Path) -> bool {
    let excluded = vec![".git".to_string()];
    let result = FolderCompare::new(first, second, &excluded).unwrap();
    result.changed_files.is_empty() && result.new_files.is_empty()
}
