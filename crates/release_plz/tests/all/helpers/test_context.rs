use std::process::Command;

use crate::helpers::gitea::CARGO_INDEX_REPO;
use assert_cmd::assert::Assert;
use cargo_metadata::{
    camino::{Utf8Path, Utf8PathBuf},
    Package,
};
use cargo_utils::LocalManifest;
use git_cmd::Repo;
use release_plz_core::{
    fs_utils::Utf8TempDir, GitBackend, GitClient, GitPr, Gitea, RepoUrl, BRANCH_PREFIX,
};
use secrecy::SecretString;

use tracing::info;

use super::{
    fake_utils,
    gitea::{gitea_address, GiteaContext},
    TEST_REGISTRY,
};

/// It contains the universe in which release-plz runs.
pub struct TestContext {
    pub gitea: GiteaContext,
    test_dir: Utf8TempDir,
    /// Release-plz git client. It's here just for code reuse.
    git_client: GitClient,
    pub repo: Repo,
}

impl TestContext {
    async fn init_context() -> Self {
        test_logs::init();
        let repo_name = fake_utils::fake_id();
        let gitea = GiteaContext::new(repo_name).await;
        let test_dir = Utf8TempDir::new().unwrap();
        info!("test directory: {:?}", test_dir.path());
        let repo_url = gitea.repo_clone_url();
        git_clone(test_dir.path(), &repo_url);

        let git_client = git_client(&repo_url, &gitea.token);

        let repo_dir = test_dir.path().join(&gitea.repo);
        let repo = configure_repo(&repo_dir, &gitea);
        Self {
            gitea,
            test_dir,
            git_client,
            repo,
        }
    }

    pub async fn new() -> Self {
        let context = Self::init_context().await;
        cargo_init(context.repo.directory());
        context.generate_cargo_lock();
        context.repo.add_all_and_commit("cargo init").unwrap();
        context.repo.git(&["push"]).unwrap();
        context
    }

    pub async fn new_workspace(crates: &[&str]) -> Self {
        let context = Self::init_context().await;
        let root_cargo_toml = {
            let quoted_crates: Vec<String> = crates.iter().map(|c| format!("\"{c}\"")).collect();
            let crates_list = quoted_crates.join(",");
            format!("[workspace]\nresolver = \"2\"\nmembers = [{crates_list}]\n")
        };
        fs_err::write(context.repo.directory().join("Cargo.toml"), root_cargo_toml).unwrap();

        for package in crates {
            let crate_dir = context.repo.directory().join(package);
            fs_err::create_dir_all(&crate_dir).unwrap();
            cargo_init(&crate_dir);
        }
        context.generate_cargo_lock();
        context.repo.add_all_and_commit("cargo init").unwrap();
        context.repo.git(&["push"]).unwrap();
        context
    }

    pub async fn merge_release_pr(&self) {
        let opened_prs = self.opened_release_prs().await;
        assert_eq!(opened_prs.len(), 1);
        self.gitea.merge_pr_retrying(opened_prs[0].number).await;
        self.repo.git(&["pull"]).unwrap();
    }

    fn generate_cargo_lock(&self) {
        assert_cmd::Command::new("cargo")
            .current_dir(self.repo.directory())
            .arg("check")
            .assert()
            .success();
    }

    pub fn run_update(&self) -> Assert {
        super::cmd::release_plz_cmd()
            .current_dir(self.repo_dir())
            .env("RUST_LOG", log_level())
            .arg("update")
            .arg("--verbose")
            .arg("--registry")
            .arg(TEST_REGISTRY)
            .assert()
    }

    pub fn run_release_pr(&self) -> Assert {
        super::cmd::release_plz_cmd()
            .current_dir(self.repo_dir())
            .env("RUST_LOG", log_level())
            .arg("release-pr")
            .arg("--verbose")
            .arg("--git-token")
            .arg(&self.gitea.token)
            .arg("--backend")
            .arg("gitea")
            .arg("--registry")
            .arg(TEST_REGISTRY)
            .arg("--output")
            .arg("json")
            .assert()
    }

    pub fn run_release(&self) -> Assert {
        super::cmd::release_plz_cmd()
            .current_dir(self.repo_dir())
            .env("RUST_LOG", log_level())
            .arg("release")
            .arg("--verbose")
            .arg("--git-token")
            .arg(&self.gitea.token)
            .arg("--backend")
            .arg("gitea")
            .arg("--registry")
            .arg(TEST_REGISTRY)
            .arg("--token")
            .arg(format!("Bearer {}", &self.gitea.token))
            .arg("--output")
            .arg("json")
            .assert()
    }

    pub fn repo_dir(&self) -> Utf8PathBuf {
        self.test_dir.path().join(&self.gitea.repo)
    }

    pub async fn opened_release_prs(&self) -> Vec<GitPr> {
        self.git_client.opened_prs(BRANCH_PREFIX).await.unwrap()
    }

    pub fn write_release_plz_toml(&self, content: &str) {
        let release_plz_toml_path = self.repo_dir().join("release-plz.toml");
        fs_err::write(release_plz_toml_path, content).unwrap();
        self.repo.add_all_and_commit("add config file").unwrap();
        self.repo.git(&["push"]).unwrap();
    }

    pub fn write_changelog(&self, content: &str) {
        let changelog_path = self.repo_dir().join("CHANGELOG.md");
        fs_err::write(changelog_path, content).unwrap();
        self.repo.add_all_and_commit("edit changelog").unwrap();
        self.repo.git(&["push"]).unwrap();
    }

    pub fn download_package(&self, dest_dir: &Utf8Path) -> Vec<Package> {
        let crate_name = &self.gitea.repo;
        release_plz_core::PackageDownloader::new([crate_name], dest_dir.as_str())
            .with_registry(TEST_REGISTRY.to_string())
            .with_cargo_cwd(self.repo_dir())
            .download()
            .unwrap()
    }
}

pub fn run_set_version(directory: &Utf8Path, change: &str) {
    let change: Vec<_> = change.split(' ').collect();
    super::cmd::release_plz_cmd()
        .current_dir(directory)
        .env("RUST_LOG", log_level())
        .arg("set-version")
        .args(&change)
        .assert();
}

fn log_level() -> String {
    if std::env::var("ENABLE_LOGS").is_ok() {
        std::env::var("RUST_LOG").unwrap_or("DEBUG,hyper=INFO".to_string())
    } else {
        "ERROR".to_string()
    }
}

fn cargo_init(crate_dir: &Utf8Path) {
    assert_cmd::Command::new("cargo")
        .current_dir(crate_dir)
        .arg("init")
        .assert()
        .success();
    edit_cargo_toml(crate_dir);
}

fn configure_repo(repo_dir: &Utf8Path, gitea: &GiteaContext) -> Repo {
    let username = gitea.user.username();
    let repo = Repo::new(repo_dir).unwrap();
    // config local user
    repo.git(&["config", "user.name", username]).unwrap();
    // set email
    repo.git(&["config", "user.email", &gitea.user.email()])
        .unwrap();

    create_cargo_config(repo_dir, username);

    repo
}

fn edit_cargo_toml(repo_dir: &Utf8Path) {
    let cargo_toml_path = repo_dir.join("Cargo.toml");
    let mut cargo_toml = LocalManifest::try_new(&cargo_toml_path).unwrap();
    let mut registry_array = toml_edit::Array::new();
    registry_array.push(TEST_REGISTRY);
    cargo_toml.data["package"]["publish"] =
        toml_edit::Item::Value(toml_edit::Value::Array(registry_array));
    cargo_toml.write().unwrap();
}

fn create_cargo_config(repo_dir: &Utf8Path, username: &str) {
    let config_dir = repo_dir.join(".cargo");
    fs_err::create_dir(&config_dir).unwrap();
    let config_file = config_dir.join("config.toml");
    let cargo_config = cargo_config(username);
    fs_err::write(config_file, cargo_config).unwrap();
}

fn cargo_config(username: &str) -> String {
    // matches the docker compose file
    let cargo_registries = format!(
        "[registry]\ndefault = \"{TEST_REGISTRY}\"\n\n[registries.{TEST_REGISTRY}]\nindex = "
    );
    // we use gitea as a cargo registry:
    // https://docs.gitea.com/usage/packages/cargo
    let gitea_index = format!(
        "\"http://{}/{}/{CARGO_INDEX_REPO}.git\"",
        gitea_address(),
        username
    );

    let config_end = r#"
[net]
git-fetch-with-cli = true
    "#;
    format!("{cargo_registries}{gitea_index}{config_end}")
}

fn git_client(repo_url: &str, token: &str) -> GitClient {
    let git_backend = GitBackend::Gitea(
        Gitea::new(
            RepoUrl::new(repo_url).unwrap(),
            SecretString::from(token.to_string()),
        )
        .unwrap(),
    );
    GitClient::new(git_backend).unwrap()
}

fn git_clone(path: &Utf8Path, repo_url: &str) {
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
