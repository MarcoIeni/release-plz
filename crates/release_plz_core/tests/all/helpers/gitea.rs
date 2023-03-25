use std::process::Command;

use fake::{Fake, StringFaker};
use serde_json::json;

pub struct User {
    username: String,
    password: String,
}

impl User {
    pub async fn create_repository(&self, repo_name: &str) {
        let client = reqwest::Client::new();
        client
            .post("http://localhost:3000/api/v1/user/repos")
            .basic_auth(&self.username, Some(&self.password))
            .json(&json!({
                "name": repo_name,
                // Automatically initialize the repository
                "auto_init": true,
            }))
            .send()
            .await
            .expect("Failed to create repository");
    }

    pub async fn repo_exists(&self, repo_name: &str) -> bool {
        let repo = self.get_repo(repo_name).await;
        repo == repo_name
    }

    /// Get the repository and return its name.
    async fn get_repo(&self, repo_name: &str) -> String {
        let repo_url = format!(
            "http://localhost:3000/api/v1/repos/{}/{}",
            self.username, repo_name
        );
        let client = reqwest::Client::new();

        let repo: Repository = client
            .get(repo_url)
            .basic_auth(&self.username, Some(&self.password))
            .send()
            .await
            .unwrap()
            .error_for_status()
            .unwrap()
            .json()
            .await
            .unwrap();
        repo.name
    }
}

#[derive(serde::Deserialize)]
struct Repository {
    name: String,
}

fn run_create_user_command(user: &User) {
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
pub fn create_user() -> User {
    let user = User {
        username: fake_id(),
        password: fake_id(),
    };
    run_create_user_command(&user);
    user
}

fn fake_id() -> String {
    const LETTERS: &str = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";
    let f = StringFaker::with(Vec::from(LETTERS), 8);
    f.fake()
}
