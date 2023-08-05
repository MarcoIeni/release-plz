//! Helpers for creating a new Gitea repository for testing.

use std::process::Command;

use serde_json::json;

use crate::helpers::{reqwest_utils::ReqwestUtils, fake_utils};

use super::{GiteaContext, GiteaUser};

impl GiteaContext {
    pub async fn new(repo: String) -> Self {
        let client = reqwest::Client::new();
        let user = create_user();
        let token = create_token(&user, &client).await;

        create_repository(&token, &repo, &client).await;

        Self {
            user,
            token,
            repo,
            client,
        }
    }
}

pub async fn create_token(user: &GiteaUser, client: &reqwest::Client) -> String {
    #[derive(serde::Deserialize)]
    struct Token {
        sha1: String,
    }

    let token: Token =
        client
        .post(format!(
            "http://localhost:3000/api/v1/users/{}/tokens",
            user.username()
        ))
        .basic_auth(user.username(), Some(&user.password()))
        .json(&json!({
            "name": user.username(),
            // edit repositories
            "scopes": ["read:repository", "write:repository", "write:user"]
        }))
        .send()
        .await
        .unwrap()
        .ok_if_2xx()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    token.sha1
}

async fn create_repository(user_token: &str, repo_name: &str, client: &reqwest::Client) {
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
        .ok_if_2xx()
        .await
        .unwrap();
}

#[tokio::test]
async fn can_create_gitea_repository() {
    let repo_name = "myrepo";
    let gitea = GiteaContext::new(repo_name.to_string()).await;
    assert!(gitea.repo_exists(repo_name).await);
}

fn run_create_user_command(user: &GiteaUser) {
    let email = format!("{}@example.com", user.username());
    Command::new("docker")
        .arg("exec")
        .arg("gitea")
        .arg("gitea")
        .arg("admin")
        .arg("user")
        .arg("create")
        .arg("--username")
        .arg(user.username())
        .arg("--password")
        .arg(user.password())
        .arg("--email")
        .arg(email)
        .arg("--must-change-password=false")
        .status()
        .expect("Failed to create user");
}

/// Create a random user and return it's username and passoword.
pub fn create_user() -> GiteaUser {
    let user = GiteaUser {
        username: fake_utils::fake_id(),
        password: "psw".to_string(),
    };
    run_create_user_command(&user);
    user
}
