use std::{
    fs,
    path::{self, Path, PathBuf},
    process::Command,
    str::FromStr,
};

use assert_cmd::assert::Assert;
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
        let repo_url = gitea.repo_clone_url();
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

    pub fn run_release_plz(&self) -> Assert {
        let log_level = if std::env::var("ENABLE_LOGS").is_ok() {
            "DEBUG,hyper=info"
        } else {
            "ERROR"
        };
        assert_cmd::Command::cargo_bin(env!("CARGO_PKG_NAME"))
            .unwrap()
            .current_dir(&self.repo_dir())
            .env("RUST_LOG", log_level)
            .arg("release-pr")
            .arg("--git-token")
            .arg(&self.gitea.token)
            .arg("--backend")
            .arg("gitea")
            .arg("--registry")
            .arg("test-registry")
            .assert()
    }

    pub fn repo_dir(&self) -> PathBuf {
        self.test_dir.path().join(&self.gitea.repo)
    }

    pub async fn opened_release_prs(&self) -> Vec<GitPr> {
        self.git_client.opened_prs("release-plz/").await.unwrap()
    }
}

fn commit_cargo_init(repo_dir: &Path, username: &str) -> Repo {
    assert_cmd::Command::new("cargo")
        .current_dir(repo_dir)
        .arg("init")
        .assert()
        .success();

    let repo = Repo::new(repo_dir).unwrap();
    // config local user
    repo.git(&["config", "user.name", username]).unwrap();
    // set email
    repo.git(&["config", "user.email", "a@example.com"])
        .unwrap();

    create_cargo_config(repo_dir);

    // Generate Cargo.lock
    assert_cmd::Command::new("cargo")
        .current_dir(repo_dir)
        .arg("check")
        .assert()
        .success();

    assert_cmd::Command::new("cargo")
        .current_dir(repo_dir)
        .arg("login")
        .arg("--registry")
        .arg("test-registry")
        .arg("random-token")
        .assert()
        .success();

    repo.add_all_and_commit("Initial commit").unwrap();
    repo.git(&["push"]).unwrap();
    repo
}

fn create_cargo_config(repo_dir: &Path) {
    // matches the docker compose file
    let cargo_config = r#"
[registries]
test-registry = { index = "http://127.0.0.1:35504/git" }

[net]
git-fetch-with-cli = true
    "#;
    let config_dir = repo_dir.join(".cargo");
    fs::create_dir(&config_dir).unwrap();
    let config_file = config_dir.join("config.toml");
    fs::write(config_file, cargo_config).unwrap();
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
