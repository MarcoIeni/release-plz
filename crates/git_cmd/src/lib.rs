//! Run git as shell shell and parse its stdout.

mod cmd;
#[cfg(feature = "test_fixture")]
pub mod test_fixture;

use std::{
    fmt,
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::{anyhow, Context};
use tracing::{debug, instrument, trace, warn, Span};

/// Repository
pub struct Repo {
    /// Directory where you want to run git operations
    directory: PathBuf,
    /// Branch name before running any git operation
    original_branch: String,
    /// Remote name before running any git operation
    original_remote: String,
}

impl Repo {
    /// Returns an error if the directory doesn't contain any commit
    #[instrument(skip_all)]
    pub fn new(directory: impl AsRef<Path>) -> anyhow::Result<Self> {
        debug!("initializing directory {:?}", directory.as_ref());

        let (current_remote, current_branch) = Self::get_current_remote_and_branch(&directory)
            .context("cannot determine current branch")?;

        Ok(Self {
            directory: directory.as_ref().to_path_buf(),
            original_branch: current_branch,
            original_remote: current_remote,
        })
    }

    pub fn directory(&self) -> &Path {
        &self.directory
    }

    fn get_current_remote_and_branch(
        directory: impl AsRef<Path>,
    ) -> anyhow::Result<(String, String)> {
        match git_in_dir(
            directory.as_ref(),
            &[
                "rev-parse",
                "--abbrev-ref",
                "--symbolic-full-name",
                "@{upstream}",
            ],
        ) {
            Ok(output) => output
                .split_once('/')
                .map(|(remote, branch)| (remote.to_string(), branch.to_string()))
                .context("cannot determine current remote and branch"),

            Err(e) => {
                let err = e.to_string();
                if err.contains("fatal: no upstream configured for branch") {
                    let branch = Self::get_current_branch(directory)?;
                    warn!("no upstream configured for branch {branch}");
                    Ok(("origin".to_string(), branch))
                } else if err.contains("fatal: ambiguous argument 'HEAD': unknown revision or path not in the working tree.") {
                    Err(anyhow!("git repository does not contain any commit."))
                } else {
                    Err(e)
                }
            }
        }
    }

    fn get_current_branch(directory: impl AsRef<Path>) -> anyhow::Result<String> {
        git_in_dir(directory.as_ref(), &["rev-parse", "--abbrev-ref", "HEAD"])
        .map_err(|e|
            if e.to_string().contains("fatal: ambiguous argument 'HEAD': unknown revision or path not in the working tree.") {
                anyhow!("git repository does not contain any commit.")
            }
            else {
                e
            }
        )
    }

    /// Check if there are uncommitted changes.
    pub fn is_clean(&self) -> anyhow::Result<()> {
        let changes = self.changes_except_typechanges()?;
        anyhow::ensure!(changes.is_empty(), "the working directory of this project has uncommitted changes. Please commit or stash these changes:\n{changes:?}");
        Ok(())
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

    pub fn changes_except_typechanges(&self) -> anyhow::Result<Vec<String>> {
        let output = self.git(&["status", "--porcelain"])?;
        let changed_files = changed_files(&output);
        Ok(changed_files)
    }

    pub fn add<T: AsRef<str>>(&self, paths: &[T]) -> anyhow::Result<()> {
        let mut args = vec!["add"];
        let paths: Vec<&str> = paths.iter().map(|p| p.as_ref()).collect();
        args.extend(paths);
        self.git(&args)?;
        Ok(())
    }

    pub fn commit(&self, message: &str) -> anyhow::Result<()> {
        self.git(&["commit", "-m", message])?;
        Ok(())
    }

    pub fn push(&self, obj: &str) -> anyhow::Result<()> {
        self.git(&["push", &self.original_remote, obj])?;
        Ok(())
    }

    pub fn fetch(&self, obj: &str) -> anyhow::Result<()> {
        self.git(&["fetch", &self.original_remote, obj])?;
        Ok(())
    }

    pub fn force_push(&self, obj: &str) -> anyhow::Result<()> {
        self.git(&["push", &self.original_remote, obj, "--force"])?;
        Ok(())
    }

    pub fn checkout_head(&self) -> anyhow::Result<()> {
        self.git(&["checkout", &self.original_branch])?;
        Ok(())
    }

    /// Branch name before running any git operation.
    /// I.e. when the [`Repo`] was created.
    pub fn original_branch(&self) -> &str {
        &self.original_branch
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
        let commit_list = self.git(&["--format=%H", "-n", &nth])?;
        let last_commit = commit_list
            .lines()
            .last()
            .context("repository has no commits")?;
        Span::current().record("nth_commit", last_commit);

        Ok(last_commit.to_string())
    }

    /// Run a git command in the repository git directory
    pub fn git(&self, args: &[&str]) -> anyhow::Result<String> {
        git_in_dir(&self.directory, args)
    }

    pub fn stash_pop(&self) -> anyhow::Result<()> {
        self.git(&["stash", "pop"])?;
        Ok(())
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
    pub fn checkout(&self, object: &str) -> anyhow::Result<()> {
        self.git(&["checkout", object])?;
        Ok(())
    }

    /// Get `nth` commit starting from `1`.
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
        let path = path
            .as_ref()
            .to_str()
            .ok_or_else(|| anyhow!("invalid path"))?;
        let commit_list = self.git(&["log", "--format=%H", "-n", &nth_str, "--", path])?;
        let mut commits = commit_list.lines();
        let last_commit = commits.nth(nth - 1).context("not enough commits")?;

        Span::current().record("nth_commit", last_commit);
        debug!("nth_commit found");
        Ok(last_commit.to_string())
    }

    pub fn current_commit_message(&self) -> anyhow::Result<String> {
        self.git(&["log", "-1", "--pretty=format:%s"])
    }

    pub fn tag(&self, name: &str) -> anyhow::Result<String> {
        self.git(&["tag", name])
    }

    /// Url of the remote when the [`Repo`] was created.
    pub fn original_remote_url(&self) -> anyhow::Result<String> {
        let param = format!("remote.{}.url", self.original_remote);
        self.git(&["config", "--get", &param])
    }

    pub fn tag_exists(&self, tag: &str) -> anyhow::Result<bool> {
        let output = self
            .git(&["tag", "-l", tag])
            .context("cannot determine if git tag exists")?;
        Ok(output.lines().count() >= 1)
    }
}

fn changed_files(output: &str) -> Vec<String> {
    output
        .lines()
        .map(|l| l.trim())
        // filter typechanges
        .filter(|l| !l.starts_with("T "))
        .filter_map(|e| e.rsplit(' ').next())
        .map(|e| e.to_string())
        .collect()
}

#[instrument]
pub fn git_in_dir(dir: &Path, args: &[&str]) -> anyhow::Result<String> {
    let args: Vec<&str> = args.iter().map(|s| s.trim()).collect();
    let output = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(&args)
        .output()
        .with_context(|| {
            format!("error while running git in directory `{dir:?}` with args `{args:?}`")
        })?;
    trace!("git {:?}: output = {:?}", args, output);
    let stdout = cmd::string_from_bytes(output.stdout)?;
    if output.status.success() {
        Ok(stdout)
    } else {
        let mut error = format!("error while running git with args `{args:?}");
        let stderr = cmd::string_from_bytes(output.stderr)?;
        if !stdout.is_empty() || !stderr.is_empty() {
            error.push(':');
        }
        if !stdout.is_empty() {
            error.push_str("\n- stdout: ");
            error.push_str(&stdout);
        }
        if !stderr.is_empty() {
            error.push_str("\n- stderr: ");
            error.push_str(&stderr);
        }
        Err(anyhow!(error))
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use tempfile::tempdir;

    use super::*;

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
            fs::write(file1, b"Hello, file1!").unwrap();
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
            fs::write(file1, b"Hello, file1!").unwrap();
            repo.add_all_and_commit(commit_message).unwrap();
        }
        assert_eq!(repo.current_commit_message().unwrap(), commit_message);
    }

    #[test]
    fn clean_project_is_recognized() {
        test_logs::init();
        let repository_dir = tempdir().unwrap();
        let repo = Repo::init(&repository_dir);
        repo.is_clean().unwrap();
    }

    #[test]
    fn dirty_project_is_recognized() {
        test_logs::init();
        let repository_dir = tempdir().unwrap();
        let repo = Repo::init(&repository_dir);
        let file1 = repository_dir.as_ref().join("file1.txt");
        fs::write(file1, b"Hello, file1!").unwrap();
        assert!(repo.is_clean().is_err());
    }

    #[test]
    fn changes_files_except_typechanges_are_detected() {
        let git_status_output = r"T CHANGELOG.md
 M README.md
A  crates
D  crates/git_cmd/CHANGELOG.md
";
        let changed_files = changed_files(git_status_output);
        assert_eq!(
            changed_files,
            vec!["README.md", "crates", "crates/git_cmd/CHANGELOG.md",]
        )
    }

    #[test]
    fn existing_tag_is_recognized() {
        test_logs::init();
        let repository_dir = tempdir().unwrap();
        let repo = Repo::init(&repository_dir);
        let file1 = repository_dir.as_ref().join("file1.txt");
        {
            fs::write(file1, b"Hello, file1!").unwrap();
            repo.add_all_and_commit("file1").unwrap();
        }
        let version = "v1.0.0";
        repo.tag(version).unwrap();
        assert!(repo.tag_exists(version).unwrap())
    }

    #[test]
    fn non_existing_tag_is_recognized() {
        test_logs::init();
        let repository_dir = tempdir().unwrap();
        let repo = Repo::init(&repository_dir);
        let file1 = repository_dir.as_ref().join("file1.txt");
        {
            fs::write(file1, b"Hello, file1!").unwrap();
            repo.add_all_and_commit("file1").unwrap();
        }
        repo.tag("v1.0.0").unwrap();
        assert!(!repo.tag_exists("v2.0.0").unwrap())
    }
}
