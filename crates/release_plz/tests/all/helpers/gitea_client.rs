use std::process::Command;

use fake::{Fake, StringFaker};
use serde_json::json;

pub struct GiteaUser {
    username: String,
    password: String,
}

/// Test context. It contains the universe in which release-plz runs.
pub struct Context {
    gitea: GiteaContext,
}

pub struct GiteaContext {
    pub user: GiteaUser,
    pub token: String,
    /// Repository name.
    repo: String,
}

impl GiteaContext {
    pub async fn new(repo: String) -> Self {
        let user = create_user();
        let token = create_token(&user).await;

        create_repository(&token, &repo).await;

        Self { user, token, repo }
    }

    pub fn repo_url(&self) -> String {
        format!(
            // if you need ssh instead of http: "ssh://git@localhost:2222/{}/{}.git",
            "http://{}:{}@localhost:3000/{}/{}.git",
            self.user.username(),
            self.user.password(),
            self.user.username(),
            self.repo
        )
    }
}

impl Context {
    pub async fn new() -> Self {
        test_logs::init();
        Self {
            gitea: GiteaContext::new("myrepo".to_string()).await,
        }
    }
}

impl GiteaContext {
    pub async fn repo_exists(&self, repo_name: &str) -> bool {
        let repo = self.get_repo(repo_name).await;
        repo == repo_name
    }

    /// Get the repository and return its name.
    async fn get_repo(&self, repo_name: &str) -> String {
        let repo_url = format!(
            "http://localhost:3000/api/v1/repos/{}/{}",
            self.user.username, repo_name
        );
        let client = reqwest::Client::new();

        let repo: Repository = client
            .get(repo_url)
            .basic_auth(&self.user.username, Some(&self.user.password))
            .send()
            .await
            .unwrap()
            .assert_2xx()
            .await
            .json()
            .await
            .unwrap();

        repo.name
    }
}

#[async_trait::async_trait]
trait Assert2xx {
    async fn assert_2xx(self) -> Self;
}

#[async_trait::async_trait]
impl Assert2xx for reqwest::Response {
    async fn assert_2xx(self) -> Self {
        let status = self.status();
        if status.is_success() {
            self.error_for_status().unwrap()
        } else {
            let response_dbg = format!("{:?}", self);
            let body = self.text().await.unwrap();
            panic!("Wrong response. Response: {}. Body: {}", response_dbg, body);
        }
    }
}

pub async fn create_token(user: &GiteaUser) -> String {
    let client = reqwest::Client::new();
    let token: Token = client
        .post(format!(
            "http://localhost:3000/api/v1/users/{}/tokens",
            user.username
        ))
        .basic_auth(&user.username, Some(&user.password))
        .json(&json!({
            "name": user.username,
            // edit repositories
            "scopes": ["read:repository", "write:repository", "write:user"]
        }))
        .send()
        .await
        .unwrap()
        .assert_2xx()
        .await
        .json()
        .await
        .unwrap();
    token.sha1
}
impl GiteaUser {
    pub fn username(&self) -> &str {
        &self.username
    }

    pub fn password(&self) -> &str {
        &self.password
    }
}

#[derive(serde::Deserialize)]
struct Repository {
    name: String,
}

#[derive(serde::Deserialize)]
struct Token {
    sha1: String,
}

fn run_create_user_command(user: &GiteaUser) {
    let email = format!("{}@example.com", user.username);
    Command::new("docker")
        .arg("exec")
        .arg("gitea")
        .arg("gitea")
        .arg("admin")
        .arg("user")
        .arg("create")
        .arg("--username")
        .arg(&user.username)
        .arg("--password")
        .arg(&user.password)
        .arg("--email")
        .arg(email)
        .arg("--must-change-password=false")
        .status()
        .expect("Failed to create user");
}

/// Create a random user and return it's username and passoword.
pub fn create_user() -> GiteaUser {
    let user = GiteaUser {
        username: fake_id(),
        password: "psw".to_string(),
    };
    run_create_user_command(&user);
    user
}

fn fake_id() -> String {
    const LETTERS: &str = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";
    let f = StringFaker::with(Vec::from(LETTERS), 8);
    f.fake()
}

pub async fn create_repository(user_token: &str, repo_name: &str) {
    let client = reqwest::Client::new();
    client
        .post("http://localhost:3000/api/v1/user/repos")
        .query(&[("token", user_token)])
        .json(&json!({
            "name": repo_name,
            // Automatically initialize the repository
            "auto_init": true,
        }))
        .send()
        .await
        .expect("Failed to create repository")
        .assert_2xx()
        .await;

}

#[tokio::test]
async fn can_create_gitea_repository() {
    let repo_name = "myrepo";
    let gitea = GiteaContext::new(repo_name.to_string()).await;
    assert!(gitea.repo_exists(repo_name).await);
}
