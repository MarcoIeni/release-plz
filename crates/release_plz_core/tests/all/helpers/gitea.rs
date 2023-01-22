use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

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

/// Create a user and return it's token.
pub async fn create_user(username: &str) -> String {
    let client = reqwest::Client::new();

    // using the sign form and not the api
    let response = client
        .post(format!("{}/user/sign_up", base_url()))
        .header("cookie", "lang=en-US; i_like_gitea=8e2779a79e7d3e28; _csrf=uBwdvQ2EKSS69kVzPIGOPI1OmoU6MTU5NDMxMTk2NzA1ODIxMjgzNw")
        .form(&CreateUserOption {
            email: "me@example.com",
            password: DEFAULT_PASSWORD,
            retype: DEFAULT_PASSWORD,
            user_name: username,
        })
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();

    // this must be called with username + password instead of a token
    // since there is no token created
    let response = client
        .post(format!("{}/users/{username}/tokens", base_api_url()))
        .basic_auth(username, Some(DEFAULT_PASSWORD))
        .json(&TokenRequest { name: "test" })
        .send()
        .await
        .unwrap();

    let token: TokenResponse = check_status_code(response, "error while creating token").await;
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

    let repo: CreateRepoResponse = check_status_code(response, "could not create a new repo").await;

    repo.html_url
}

/// creates a branch based on main
pub async fn create_branch(token: &str, repo: &str, owner: &str, new_branch_name: &str) {
    let response = do_gitea_request(
        format!("repos/{owner}/{repo}/branches").as_str(),
        token,
        &CreateBranchRequest { new_branch_name },
    )
    .await;

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
