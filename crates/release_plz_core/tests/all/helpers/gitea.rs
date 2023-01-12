use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Default, Serialize)]
pub struct CreateUserOption<'a> {
    pub email: &'a str,
    pub password: &'a str,
    pub username: &'a str,
}

#[derive(Clone, Debug, PartialEq, Default, Deserialize)]
struct TokenResponse {
    sha1: String,
}

/// Create a user and return it's token.
async fn create_user() -> String {
    let client = reqwest::Client::new();
    let username = "me";
    let admin_pwd: Option<String> = None;
    let user_pwd = "password";
    client
        .post(format!("{}/admin/users", base_url()))
        .basic_auth("root", admin_pwd.clone())
        .json(&CreateUserOption {
            email: "me@example.com",
            password: user_pwd,
            username,
        })
        .send()
        .await
        .unwrap();

    let token: TokenResponse = client
        .post(format!("{}/users/{username}/tokens", base_url()))
        .basic_auth(username, Some(user_pwd))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    token.sha1
}

fn base_url() -> String {
    "http://localhost:3000/api/v1".to_string()
}
