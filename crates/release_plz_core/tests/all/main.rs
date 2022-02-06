mod update;

use std::{
    path::{Path, PathBuf},
    process::Command,
};

use git_cmd::Repo;
use release_plz_core::CARGO_TOML;

fn join_cargo_toml(project: &Path) -> PathBuf {
    project.join(CARGO_TOML)
}

fn init_project(project: &Path) {
    Command::new("cargo")
        .arg("new")
        .arg(project)
        .output()
        .unwrap();

    Repo::init(project);
}
