use release_plz_core::Gitea;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::process::Command;

pub const DEFAULT_PASSWORD: &str = "password";

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

#[derive(Clone, Debug, PartialEq, Default, Deserialize)]
pub struct PullRequest {
    pub html_url: String,
    pub title: String,
}

fn create_user_docker_exec(username: &str) {
    let response = Command::new("docker")
        .arg("exec")
        .arg("gitea")
        .arg("gitea")
        .arg("admin")
        .arg("user")
        .arg("create")
        .arg("--username")
        .arg(username)
        .arg("--password")
        .arg(DEFAULT_PASSWORD)
        .arg("--email")
        .arg("me@example.com")
        .output()
        .unwrap();

    dbg!(response);
}

/// Create a user and return it's token.
pub async fn create_user(username: &str) -> String {
    create_user_docker_exec(username);
    // this must be called with username + password instead of a token
    // since there is no token created
    let client = reqwest::Client::new();
    let response = client
        .post(format!("{}/users/{username}/tokens", base_api_url()))
        .basic_auth(username, Some(DEFAULT_PASSWORD))
        .json(&TokenRequest { name: "test" })
        .send()
        .await
        .unwrap();

    let token: TokenResponse = deserialize_response(response, "error while creating token").await;
    token.sha1
}

/// create a repo and returns its url
pub async fn create_repo(token: &str, repo_name: &str) -> String {
    let response = do_gitea_request(
        "user/repos",
        token,
        &CreateRepoRequest {
            name: repo_name,
            auto_init: true,
        },
    )
    .await;

    let repo: CreateRepoResponse =
        deserialize_response(response, "could not create a new repo").await;

    repo.html_url
}

/// list all open pull requests for the repo
pub async fn list_pull_requests(gitea_info: &Gitea) -> Vec<PullRequest> {
    let client = reqwest::Client::new();
    let response = client
        .get(format!(
            "{}/repos/{}/{}/pulls?state=open",
            base_api_url(),
            gitea_info.owner,
            gitea_info.repo,
        ))
        .header("accept", "application/json")
        .send()
        .await
        .unwrap();

    deserialize_response(response, "could not list all pull requests").await
}

fn base_api_url() -> String {
    format!("{}/api/v1", base_url())
}

pub fn base_url() -> String {
    "http://localhost:3000".to_string()
}

pub fn git_cred_url(username: &str, repo_name: &str) -> String {
    format!("http://{username}:{DEFAULT_PASSWORD}@localhost:3000/{username}/{repo_name}")
}

pub fn repo_url(username: &str, repo: &str) -> String {
    format!("{}/{username}/{repo}", base_url())
}

async fn do_gitea_request<T: Serialize>(api: &str, token: &str, request: &T) -> reqwest::Response {
    let client = reqwest::Client::new();
    client
        .post(format!("{}/{api}", base_api_url()))
        .header(reqwest::header::AUTHORIZATION, format!("token {token}"))
        .json(request)
        .send()
        .await
        .unwrap()
}

async fn deserialize_response<T: DeserializeOwned>(
    response: reqwest::Response,
    error_message: &str,
) -> T {
    let status = response.status();
    if status != 201 && status != 200 {
        match response.text().await {
            Ok(txt) => panic!("{error_message}, status_code: {status}, response: {txt}"),
            Err(e) => panic!(
                "{error_message}, status_code: {status}, could not retrieve response as text: {e}"
            ),
        }
    }
    response.json().await.unwrap()
}
