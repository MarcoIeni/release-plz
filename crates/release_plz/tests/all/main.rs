use std::{
    fs, io,
    path::{Path, PathBuf},
    process::{Command, Output},
};

use fs_extra::dir;
use tempfile::tempdir;

fn join_cargo_toml(project: &Path) -> PathBuf {
    project.join("Cargo.toml")
}

fn git_in_dir(dir: &Path, args: &[&str]) -> io::Result<Output> {
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

    release_plz::update(
        &join_cargo_toml(&local_project),
        &join_cargo_toml(&remote_project.as_ref().join("myproject")),
    )
    .unwrap();

    let are_dir_different = dir_diff::is_different(
        &local_project_dir.as_ref().join("myproject"),
        remote_project.as_ref().join("myproject"),
    )
    .unwrap();
    // the update should have not changed anything
    assert!(!are_dir_different);
}
