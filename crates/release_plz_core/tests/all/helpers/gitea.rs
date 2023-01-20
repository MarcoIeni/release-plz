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
}

#[derive(Clone, Debug, PartialEq, Default, Deserialize)]
struct CreateRepoResponse {
    html_url: String,
}

/// Create a user and return it's token.
pub async fn create_user() -> String {
    let client = reqwest::Client::new();
    let username = "me";
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

    let token: TokenResponse = client
        .post(format!("{}/users/{username}/tokens", base_api_url()))
        .basic_auth(username, Some(user_pwd))
        //TODO name must be unique
        .json(&TokenRequest { name: "test" })
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    dbg!(&token);

    token.sha1
}

/// create a repo and returns its url
pub async fn create_repo(token: &str, repo_name: &str) -> String {
    let client = reqwest::Client::new();
    let repo: CreateRepoResponse = client
        .post(format!("{}/user/repos", base_api_url()))
        .header(reqwest::header::AUTHORIZATION, format!("token {token}"))
        .json(&CreateRepoRequest { name: repo_name })
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    repo.html_url
}

fn base_api_url() -> String {
    format!("{}/api/v1", base_url())
}

fn base_url() -> String {
    "http://localhost:3000".to_string()
}
