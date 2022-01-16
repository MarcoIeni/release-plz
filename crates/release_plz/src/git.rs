use std::{
    fmt, io,
    path::{Path, PathBuf},
    process::{Command, Output},
};

use anyhow::{anyhow, Context};
use tracing::{debug, instrument, Span};

/// Repository
pub struct Repo {
    /// Repository root directory
    directory: PathBuf,
    default_branch: String,
}

impl Drop for Repo {
    fn drop(&mut self) {
        let _err = self.checkout(&self.default_branch);
    }
}

impl Repo {
    /// Returns an error if the directory doesn't contain any commit
    pub fn new(directory: impl AsRef<Path>) -> anyhow::Result<Self> {
        let current_branch = Self::get_current_branch(&directory)?;

        Ok(Self {
            directory: directory.as_ref().to_path_buf(),
            default_branch: current_branch,
        })
    }

    fn get_current_branch(directory: impl AsRef<Path>) -> anyhow::Result<String> {
        let current_branch =
            Self::git_in_dir(directory.as_ref(), &["rev-parse", "--abbrev-ref", "HEAD"])?;
        stdout(current_branch).map_err(|e|
            if e.to_string().contains("fatal: ambiguous argument 'HEAD': unknown revision or path not in the working tree.") {
                anyhow!("git repository does not contain any commit.")
            }
            else {
                e
            }
        )
    }

    pub fn checkout_new_branch(&self, branch: &str) -> anyhow::Result<()> {
        self.git(&["checkout", "-b", branch])?;
        Ok(())
    }

    pub fn add_all_and_commit(&self, message: &str) -> anyhow::Result<()> {
        self.git(&["add", "."])?;
        self.git(&["commit", "-m", message])?;
        Ok(())
    }

    pub fn push(&self, branch: &str) -> anyhow::Result<()> {
        self.git(&["push", "origin", branch])?;
        Ok(())
    }

    pub fn checkout_head(&self) -> anyhow::Result<()> {
        self.git(&["checkout", &self.default_branch])?;
        Ok(())
    }

    #[instrument(skip(self))]
    fn current_commit(&self) -> anyhow::Result<String> {
        self.nth_commit(1)
    }

    #[instrument(skip(self))]
    fn previous_commit(&self) -> anyhow::Result<String> {
        self.nth_commit(2)
    }

    #[instrument(
        skip(self)
        fields(
            nth_commit = tracing::field::Empty,
        )
    )]
    fn nth_commit(&self, nth: usize) -> anyhow::Result<String> {
        let nth = nth.to_string();
        let output = self.git(&["--format=%H", "-n", &nth])?;
        let commit_list = stdout(output)?;
        let last_commit = commit_list
            .lines()
            .last()
            .context("repository has no commits")?;
        Span::current().record("nth_commit", &last_commit);

        Ok(last_commit.to_string())
    }

    #[instrument]
    fn git_in_dir(dir: &Path, args: &[&str]) -> io::Result<Output> {
        let args: Vec<&str> = args.iter().map(|s| s.trim()).collect();
        let output = Command::new("git").arg("-C").arg(dir).args(args).output();
        debug!("git output = {:?}", output);
        output
    }

    /// Run a git command in the repository git directory
    fn git(&self, args: &[&str]) -> io::Result<Output> {
        Self::git_in_dir(&self.directory, args)
    }

    /// Checkout to the latest commit.
    pub fn checkout_last_commit_at_path(&self, path: &Path) -> anyhow::Result<()> {
        let previous_commit = self.last_commit_at_path(path)?;
        self.checkout(&previous_commit)?;
        Ok(())
    }

    fn last_commit_at_path(&self, path: &Path) -> anyhow::Result<String> {
        self.nth_commit_at_path(1, path)
    }

    fn previous_commit_at_path(&self, path: &Path) -> anyhow::Result<String> {
        self.nth_commit_at_path(2, path)
    }

    pub fn checkout_previous_commit_at_path(&self, path: &Path) -> anyhow::Result<()> {
        let commit = self.previous_commit_at_path(path)?;
        self.checkout(&commit)?;
        Ok(())
    }

    #[instrument(skip(self))]
    fn checkout(&self, object: &str) -> io::Result<()> {
        self.git(&["checkout", object])?;
        Ok(())
    }

    #[instrument(
        skip(self)
        fields(
            nth_commit = tracing::field::Empty,
        )
    )]
    fn nth_commit_at_path(
        &self,
        nth: usize,
        path: impl AsRef<Path> + fmt::Debug,
    ) -> anyhow::Result<String> {
        let nth_str = nth.to_string();
        let path = path.as_ref().to_str().ok_or(anyhow!("invalid path"))?;
        let output = self.git(&["log", "--format=%H", "-n", &nth_str, path])?;
        let commit_list = stdout(output)?;
        let mut commits = commit_list.lines();
        // check if there are enough commits
        for _ in 1..nth {
            // discard previous commits
            commits.next().ok_or(anyhow!("not enough commits"))?;
        }
        let last_commit = commits.next().context("repository has no commits")?;

        Span::current().record("nth_commit", &last_commit);
        debug!("nth_commit found");
        Ok(last_commit.to_string())
    }

    pub fn current_commit_message(&self) -> anyhow::Result<String> {
        let output = self.git(&["log", "-1", "--pretty=format:%s"])?;
        stdout(output)
    }
}

#[instrument(skip_all)]
fn stdout(output: Output) -> anyhow::Result<String> {
    debug!("output: {:?}", output);
    let stdout = String::from_utf8(output.stdout)?;
    Ok(stdout)
}

#[cfg(test)]
mod tests {
    use std::fs;
    use tempfile::tempdir;

    use super::*;

    impl Repo {
        #[instrument(skip(directory))]
        fn init(directory: impl AsRef<Path>) -> Self {
            let directory = directory.as_ref();
            Self::git_in_dir(directory, &["init"]).unwrap();

            // configure author
            Self::git_in_dir(directory, &["config", "user.name", "author_name"]).unwrap();
            Self::git_in_dir(directory, &["config", "user.email", "author@example.com"]).unwrap();

            fs::write(directory.join("README.md"), "# my awesome project").unwrap();
            Self::git_in_dir(directory, &["add", "."]).unwrap();
            Self::git_in_dir(directory, &["commit", "-m", "add README"]).unwrap();
            debug!("repo initialized at {:?}", directory);
            Self::new(directory).unwrap()
        }
    }

    #[test]
    fn inexistent_previous_commit_detected() {
        let repository_dir = tempdir().unwrap();
        let repo = Repo::init(&repository_dir);
        let file1 = repository_dir.as_ref().join("file1.txt");
        repo.checkout_previous_commit_at_path(&file1).unwrap_err();
    }

    #[test]
    fn previous_commit_is_retrieved() {
        test_logs::init();
        let repository_dir = tempdir().unwrap();
        let repo = Repo::init(&repository_dir);
        let file1 = repository_dir.as_ref().join("file1.txt");
        let file2 = repository_dir.as_ref().join("file2.txt");
        {
            fs::write(&file2, b"Hello, file2!-1").unwrap();
            repo.add_all_and_commit("file2-1").unwrap();
            fs::write(&file1, b"Hello, file1!").unwrap();
            repo.add_all_and_commit("file1").unwrap();
            fs::write(&file2, b"Hello, file2!-2").unwrap();
            repo.add_all_and_commit("file2-2").unwrap();
        }
        repo.checkout_previous_commit_at_path(&file2).unwrap();
        assert_eq!(repo.current_commit_message().unwrap(), "file2-1");
    }

    #[test]
    fn current_commit_is_retrieved() {
        test_logs::init();
        let repository_dir = tempdir().unwrap();
        let repo = Repo::init(&repository_dir);
        let file1 = repository_dir.as_ref().join("file1.txt");
        let commit_message = "file1 message";
        {
            fs::write(&file1, b"Hello, file1!").unwrap();
            repo.add_all_and_commit(commit_message).unwrap();
        }
        assert_eq!(repo.current_commit_message().unwrap(), commit_message);
    }
}
