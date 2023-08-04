mod helpers;
mod release_pr;
mod update;

use std::{path::Path, process::Command};

use git_cmd::Repo;

fn init_project(project: &Path) {
    Command::new("cargo")
        .arg("new")
        .arg(project)
        .output()
        .unwrap();

    Repo::init(project);
}
