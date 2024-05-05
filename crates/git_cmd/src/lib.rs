//! Run git as shell shell and parse its stdout.

mod cmd;
#[cfg(feature = "test_fixture")]
pub mod test_fixture;

use std::{path::Path, process::Command};

use anyhow::{anyhow, Context};
use camino::{Utf8Path, Utf8PathBuf};
use tracing::{debug, instrument, trace, warn, Span};

/// Repository
#[derive(Debug)]
pub struct Repo {
    /// Directory where you want to run git operations
    directory: Utf8PathBuf,
    /// Branch name before running any git operation
    original_branch: String,
    /// Remote name before running any git operation
    original_remote: String,
}

impl Repo {
    /// Returns an error if the directory doesn't contain any commit
    #[instrument(skip_all)]
    pub fn new(directory: impl AsRef<Utf8Path>) -> anyhow::Result<Self> {
        debug!("initializing directory {:?}", directory.as_ref());

        let (current_remote, current_branch) = Self::get_current_remote_and_branch(&directory)
            .context("cannot determine current branch")?;

        Ok(Self {
            directory: directory.as_ref().to_path_buf(),
            original_branch: current_branch,
            original_remote: current_remote,
        })
    }

    pub fn directory(&self) -> &Utf8Path {
        &self.directory
    }

    fn get_current_remote_and_branch(
        directory: impl AsRef<Utf8Path>,
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
                    let branch = get_current_branch(directory)?;
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

    /// Check if there are uncommitted changes.
    pub fn is_clean(&self) -> anyhow::Result<()> {
        let changes = self.changes_except_typechanges()?;
        anyhow::ensure!(changes.is_empty(), "the working directory of this project has uncommitted changes. If these files are both committed and in .gitignore, either delete them or remove them from .gitignore. Otherwise, please commit or stash these changes:\n{changes:?}");
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

    /// Get the list of changed files.
    /// `filter` is applied for each line of `git status --porcelain`.
    /// Only changes for which `filter` returns true are returned.
    pub fn changes(&self, filter: impl FnMut(&&str) -> bool) -> anyhow::Result<Vec<String>> {
        let output = self.git(&["status", "--porcelain"])?;
        let changed_files = changed_files(&output, filter);
        Ok(changed_files)
    }

    pub fn changes_except_typechanges(&self) -> anyhow::Result<Vec<String>> {
        self.changes(|line| !line.starts_with("T "))
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

    pub fn commit_signed(&self, message: &str) -> anyhow::Result<()> {
        self.git(&["commit", "-s", "-m", message])?;
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

    #[instrument(skip(self))]
    pub fn checkout_head(&self) -> anyhow::Result<()> {
        self.checkout(&self.original_branch)?;
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
    pub fn checkout_last_commit_at_paths(&self, paths: &[&Path]) -> anyhow::Result<()> {
        let previous_commit = self.last_commit_at_paths(paths)?;
        self.checkout(&previous_commit)?;
        Ok(())
    }

    fn last_commit_at_paths(&self, paths: &[&Path]) -> anyhow::Result<String> {
        self.nth_commit_at_paths(1, paths)
    }

    fn previous_commit_at_paths(&self, paths: &[&Path]) -> anyhow::Result<String> {
        self.nth_commit_at_paths(2, paths)
    }

    pub fn checkout_previous_commit_at_paths(&self, paths: &[&Path]) -> anyhow::Result<()> {
        let commit = self.previous_commit_at_paths(paths)?;
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
    fn nth_commit_at_paths(&self, nth: usize, paths: &[&Path]) -> anyhow::Result<String> {
        let nth_str = nth.to_string();

        let git_args = {
            let mut git_args = vec!["log", "--format=%H", "-n", &nth_str, "--"];
            for p in paths {
                let path = p.to_str().expect("invalid path");
                git_args.push(path);
            }
            git_args
        };

        let commit_list = self.git(&git_args)?;
        let mut commits = commit_list.lines();
        let last_commit = commits.nth(nth - 1).context("not enough commits")?;

        Span::current().record("nth_commit", last_commit);
        debug!("nth_commit found");
        Ok(last_commit.to_string())
    }

    pub fn current_commit_message(&self) -> anyhow::Result<String> {
        self.git(&["log", "-1", "--pretty=format:%B"])
    }

    /// Get the SHA1 of the current HEAD.
    pub fn current_commit_hash(&self) -> anyhow::Result<String> {
        self.git(&["log", "-1", "--pretty=format:%H"])
            .context("can't determine current commit hash")
    }

    /// Create a git tag
    pub fn tag(&self, name: &str, message: &str) -> anyhow::Result<String> {
        self.git(&["tag", "-m", message, name])
    }

    /// Get the commit hash of the given tag
    pub fn get_tag_commit(&self, tag: &str) -> Option<String> {
        self.git(&["rev-list", "-n", "1", tag]).ok()
    }

    /// Check if a commit comes before another one.
    ///
    /// ## Example
    ///
    /// For this git log:
    /// ```txt
    /// commit d6ec399b80d44bf9c4391e4a9ead8482faa9bffd
    /// commit e880d8786cb16aa9a3f258e7503932445d708df7
    /// ```
    ///
    /// `git.is_ancestor("e880d8786cb16aa9a3f258e7503932445d708df7", "d6ec399b80d44bf9c4391e4a9ead8482faa9bffd")` returns true.
    pub fn is_ancestor(&self, maybe_ancestor_commit: &str, descendant_commit: &str) -> bool {
        self.git(&[
            "merge-base",
            "--is-ancestor",
            maybe_ancestor_commit,
            descendant_commit,
        ])
        .is_ok()
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

pub fn is_file_ignored(repo_path: &Utf8Path, file: &Utf8Path) -> bool {
    let file = file.as_str();

    git_in_dir(repo_path, &["check-ignore", "--no-index", file]).is_ok()
}

fn changed_files(output: &str, filter: impl FnMut(&&str) -> bool) -> Vec<String> {
    output
        .lines()
        .map(|l| l.trim())
        // filter typechanges
        .filter(filter)
        .filter_map(|e| e.rsplit(' ').next())
        .map(|e| e.to_string())
        .collect()
}

#[instrument]
pub fn git_in_dir(dir: &Utf8Path, args: &[&str]) -> anyhow::Result<String> {
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
        let mut error =
            format!("error while running git in directory `{dir:?}` with args `{args:?}");
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

/// Get the name of the current branch.
fn get_current_branch(directory: impl AsRef<Utf8Path>) -> anyhow::Result<String> {
    git_in_dir(directory.as_ref(), &["rev-parse", "--abbrev-ref", "HEAD"]).map_err(|e| {
        if e.to_string().contains(
            "fatal: ambiguous argument 'HEAD': unknown revision or path not in the working tree.",
        ) {
            anyhow!("git repository does not contain any commit.")
        } else {
            e
        }
    })
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
        repo.checkout_previous_commit_at_paths(&[&file1])
            .unwrap_err();
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
        repo.checkout_previous_commit_at_paths(&[&file2]).unwrap();
        assert_eq!(repo.current_commit_message().unwrap(), "file2-1");
    }

    #[test]
    fn current_commit_is_retrieved() {
        test_logs::init();
        let repository_dir = tempdir().unwrap();
        let repo = Repo::init(&repository_dir);
        let file1 = repository_dir.as_ref().join("file1.txt");

        let commit_message = r"feat: my feature

        message

        footer: small note";

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
        let changed_files = changed_files(git_status_output, |line| !line.starts_with("T "));
        // `CHANGELOG.md` is ignored because it's a typechange
        let expected_changed_files = vec!["README.md", "crates", "crates/git_cmd/CHANGELOG.md"];
        assert_eq!(changed_files, expected_changed_files);
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
        repo.tag(version, "test").unwrap();
        assert!(repo.tag_exists(version).unwrap());
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
        repo.tag("v1.0.0", "test").unwrap();
        assert!(!repo.tag_exists("v2.0.0").unwrap());
    }
}
