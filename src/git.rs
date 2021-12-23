use std::{
    io,
    path::PathBuf,
    process::{Command, Output},
};

use anyhow::Context;

struct Repo {
    /// Repository root directory
    directory: PathBuf,
}

impl Repo {
    fn previous_commit(&self) -> anyhow::Result<String> {
        let output = self.git(&["--format=\"%H\"", "-n", "2"])?;
        let last_two_commits = String::from_utf8(output.stdout)?;
        let previous_commit = last_two_commits
            .lines()
            .last()
            .context("repository has no commits")?;
        Ok(previous_commit.to_string())
    }

    /// Run a git command in the repository git directory
    fn git(&self, args: &[&str]) -> io::Result<Output> {
        Command::new("git")
            .arg("-C")
            .arg(&self.directory)
            .args(args)
            .output()
    }

    /// Checkout to the latest commit. I.e. go back in history of 1 commit.
    pub fn checkout_last_commit(&self) -> anyhow::Result<()> {
        let previous_commit = self.previous_commit()?;
        self.git(&["checkout", &previous_commit])?;
        Ok(())
    }

    pub fn get_edited_file(&self, commit: &str) -> anyhow::Result<Vec<String>> {
        let output = self.git(&["diff-tree", "--no-commit-id", "--name-only", "-r", commit])?;
        let files = String::from_utf8(output.stdout)?;
        let files = files.lines().map(String::from).collect();
        Ok(files)
    }
}
