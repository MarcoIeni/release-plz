use std::{
    path::{self, Path, PathBuf},
    process::Command,
    str::FromStr,
};

use git_cmd::Repo;
use release_plz_core::{GitBackend, GitClient, GitPr, Gitea, RepoUrl};
use secrecy::SecretString;
use tempfile::TempDir;
use tracing::info;

use super::{fake_utils, gitea::GiteaContext};

/// It contains the universe in which release-plz runs.
pub struct TestContext {
    pub gitea: GiteaContext,
    test_dir: TempDir,

    /// Release-plz git client. It's here just for code reuse.
    git_client: GitClient,
}

impl TestContext {
    pub async fn new() -> Self {
        test_logs::init();
        let repo_name = fake_utils::fake_id();
        let gitea = GiteaContext::new(repo_name).await;
        let test_dir = tempfile::tempdir().unwrap();
        info!("test directory: {:?}", test_dir.path());
        let repo_url = gitea.repo_url();
        git_clone(test_dir.path(), &repo_url);

        let git_client = git_client(&repo_url, &gitea.token);

        let repo_dir = test_dir.path().join(&gitea.repo);
        let _repo = commit_cargo_init(&repo_dir, gitea.user.username());
        Self {
            gitea,
            test_dir,
            git_client,
        }
    }

    pub fn repo_dir(&self) -> PathBuf {
        self.test_dir.path().join(&self.gitea.repo)
    }

    pub async fn opened_release_prs(&self) -> Vec<GitPr> {
        self.git_client.opened_prs("release-plz/").await.unwrap()
    }
}

fn commit_cargo_init(repo_dir: &Path, username: &str) -> Repo {
    let result = Command::new("cargo")
        .current_dir(repo_dir)
        .arg("init")
        .output()
        .unwrap();
    assert!(result.status.success());

    let repo = Repo::new(repo_dir).unwrap();
    // config local user
    repo.git(&["config", "user.name", username]).unwrap();
    // set email
    repo.git(&["config", "user.email", "a@example.com"])
        .unwrap();

    repo.add_all_and_commit("Initial commit").unwrap();
    // TODO: git push
    repo
}

fn git_client(repo_url: &str, token: &str) -> GitClient {
    let git_backend = GitBackend::Gitea(
        Gitea::new(
            RepoUrl::new(repo_url).unwrap(),
            SecretString::from_str(token).unwrap(),
        )
        .unwrap(),
    );
    GitClient::new(git_backend).unwrap()
}

fn git_clone(path: &path::Path, repo_url: &str) {
    let result = Command::new("git")
        .current_dir(path)
        .arg("clone")
        .arg(repo_url)
        .spawn()
        .unwrap()
        .wait()
        .unwrap();
    assert!(result.success());
}
