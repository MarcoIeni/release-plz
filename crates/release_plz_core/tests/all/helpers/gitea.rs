use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Default, Serialize)]
pub struct CreateUserOption<'a> {
    pub email: &'a str,
    pub password: &'a str,
    pub retype: &'a str,
    pub user_name: &'a str,
}

#[derive(Clone, Debug, PartialEq, Default, Serialize)]
struct TokenRequest<'a> {
    name: &'a str,
}

#[derive(Clone, Debug, PartialEq, Default, Deserialize)]
struct TokenResponse {
    sha1: String,
}

#[derive(Clone, Debug, PartialEq, Default, Serialize)]
struct CreateRepoRequest<'a> {
    name: &'a str,
    auto_init: bool,
}

#[derive(Clone, Debug, PartialEq, Default, Deserialize)]
struct CreateRepoResponse {
    html_url: String,
}

#[derive(Clone, Debug, PartialEq, Default, Serialize)]
struct CreateBranchRequest<'a> {
    new_branch_name: &'a str,
}

/// Create a user and return it's token.
pub async fn create_user(username: &str) -> String {
    let client = reqwest::Client::new();
    let user_pwd = "password";

    let response = client
        .post(format!("{}/user/sign_up", base_url()))
        .header("cookie", "lang=en-US; i_like_gitea=8e2779a79e7d3e28; _csrf=uBwdvQ2EKSS69kVzPIGOPI1OmoU6MTU5NDMxMTk2NzA1ODIxMjgzNw")
        .form(&CreateUserOption {
            email: "me@example.com",
            password: user_pwd,
            retype: user_pwd,
            user_name: username,
        })
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    dbg!(response);

    let response = client
        .post(format!("{}/users/{username}/tokens", base_api_url()))
        .basic_auth(username, Some(user_pwd))
        .json(&TokenRequest { name: "test" })
        .send()
        .await
        .unwrap();

    let token: TokenResponse = check_status_code(response, "error while creating token").await;
    token.sha1
}

/// create a repo and returns its url
pub async fn create_repo(token: &str, repo_name: &str) -> String {
    let client = reqwest::Client::new();
    let response = client
        .post(format!("{}/user/repos", base_api_url()))
        .header(reqwest::header::AUTHORIZATION, format!("token {token}"))
        .json(&CreateRepoRequest {
            name: repo_name,
            auto_init: true,
        })
        .send()
        .await
        .unwrap();

    let repo: CreateRepoResponse = check_status_code(response, "could not create a new repo").await;

    repo.html_url
}

/// creates a branch based on main
pub async fn create_branch(token: &str, repo: &str, owner: &str, new_branch_name: &str) {
    let client = reqwest::Client::new();
    let response = client
        .post(format!("{}/repos/{owner}/{repo}/branches", base_api_url()))
        .header(reqwest::header::AUTHORIZATION, format!("token {token}"))
        .json(&CreateBranchRequest { new_branch_name })
        .send()
        .await
        .unwrap();

    let repo: serde_json::Value =
        check_status_code(response, "could not create a new branch based on main").await;
    dbg!(repo);
}

fn base_api_url() -> String {
    format!("{}/api/v1", base_url())
}

pub fn base_url() -> String {
    "http://localhost:3000".to_string()
}

async fn check_status_code<T: DeserializeOwned>(
    response: reqwest::Response,
    error_message: &str,
) -> T {
    let status = response.status();
    if status != 201 {
        match response.text().await {
            Ok(txt) => panic!("{error_message}, status_code: {status}, response: {txt}"),
            Err(e) => panic!(
                "{error_message}, status_code: {status}, could not retrieve response as text: {e}"
            ),
        }
    }
    response.json().await.unwrap()
}
