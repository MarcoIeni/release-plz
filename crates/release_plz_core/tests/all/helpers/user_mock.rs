use std::{fs, path::Path};

use git_cmd::git_in_dir;

pub fn add_feature(project: &Path, message: &str) {
    fs::write(project.join("src").join("lib.rs"), "fn hello(){}").unwrap();
    git_in_dir(project, &["add", "."]).unwrap();
    let commit_message = format!("feat: {message}");
    git_in_dir(project, &["commit", "-m", &commit_message]).unwrap();
}
