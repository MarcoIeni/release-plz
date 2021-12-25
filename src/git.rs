use std::{
    fs, io,
    path::{Path, PathBuf},
    process::{Command, Output},
};

use anyhow::Context;

/// Repository
pub struct Repo {
    /// Repository root directory
    directory: PathBuf,
    current_branch: String,
}

impl Repo {
    pub fn new(directory: impl AsRef<Path>) -> anyhow::Result<Self> {
        let current_branch =
            Self::git_in_dir(directory.as_ref(), &["rev-parse", "--abbrev-ref", "HEAD"])?;
        let current_branch = stdout(current_branch)?;

        Ok(Self {
            directory: directory.as_ref().to_path_buf(),
            current_branch,
        })
    }

    fn current_branch(&self) -> &str {
        &self.current_branch
    }

    fn current_commit(&self) -> anyhow::Result<String> {
        self.nth_commit(1)
    }

    fn previous_commit(&self) -> anyhow::Result<String> {
        self.nth_commit(2)
    }

    fn nth_commit(&self, nth: usize) -> anyhow::Result<String> {
        let nth = nth.to_string();
        let output = self.git(&["--format=\"%H\"", "-n", &nth])?;
        let commit_list = stdout(output)?;
        let previous_commit = commit_list
            .lines()
            .last()
            .context("repository has no commits")?;
        Ok(previous_commit.to_string())
    }

    fn git_in_dir(dir: &Path, args: &[&str]) -> io::Result<Output> {
        Command::new("git").arg("-C").arg(dir).args(args).output()
    }

    /// Run a git command in the repository git directory
    fn git(&self, args: &[&str]) -> io::Result<Output> {
        Self::git_in_dir(&self.directory, args)
    }

    /// Checkout to the latest commit. I.e. go back in history of 1 commit.
    pub fn checkout_last_commit(&self) -> anyhow::Result<()> {
        let previous_commit = self.previous_commit()?;
        self.git(&["checkout", &previous_commit])?;
        Ok(())
    }

    /// Return the list of edited files of that commit. Absolute Path.
    pub fn edited_file_in_current_commit(&self) -> anyhow::Result<Vec<PathBuf>> {
        let commit = &self.current_commit()?;
        let output = self.git(&["diff-tree", "--no-commit-id", "--name-only", "-r", commit])?;
        let files = stdout(output)?;
        let files: Result<Vec<PathBuf>, io::Error> = files.lines().map(fs::canonicalize).collect();
        Ok(files?)
    }

    fn previous_commit_at_path(&self, path: impl AsRef<str>) -> anyhow::Result<String> {
        self.nth_commit_at_path(2, path)
    }

    fn nth_commit_at_path(&self, nth: usize, path: impl AsRef<str>) -> anyhow::Result<String> {
        let nth = nth.to_string();
        let output = self.git(&["log", "-p", path.as_ref(), "--format=\"%H\"", "-n", &nth])?;
        let commit_list = stdout(output)?;
        let previous_commit = commit_list
            .lines()
            .last()
            .context("repository has no commits")?;
        Ok(previous_commit.to_string())
    }

    /// Return the list of edited files of that commit. Absolute Path.
    fn edited_file(&self, commit: &str) -> anyhow::Result<Vec<PathBuf>> {
        let output = self.git(&["diff-tree", "--no-commit-id", "--name-only", "-r", commit])?;
        let files = stdout(output)?;
        let files: Result<Vec<PathBuf>, io::Error> = files.lines().map(fs::canonicalize).collect();
        Ok(files?)
    }

    pub fn get_current_commit_message(&self) -> anyhow::Result<String> {
        let output = self.git(&["log", " -1", "--pretty=format:%s"])?;
        stdout(output)
    }
}

fn stdout(output: Output) -> anyhow::Result<String> {
    let stdout = String::from_utf8(output.stdout)?;
    Ok(stdout)
}
