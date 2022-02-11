use std::{fs, path::Path};

use git_cmd::git_in_dir;

pub fn add_feature(project: &Path) {
    fs::write(project.join("src").join("lib.rs"), "do amazing things").unwrap();
    git_in_dir(project, &["add", "."]).unwrap();
    git_in_dir(project, &["commit", "-m", "feat: do awesome stuff"]).unwrap();
}
