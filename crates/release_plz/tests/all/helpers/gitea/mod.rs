mod gitea_client;
mod gitea_new;

pub use gitea_client::*;
pub use gitea_new::*;

pub struct GiteaUser {
    username: String,
    password: String,
}

pub struct GiteaContext {
    pub user: GiteaUser,
    pub token: String,
    /// Repository name.
    pub repo: String,
    client: reqwest::Client,
}

fn gitea_endpoint(endpoint: &str) -> String {
    let api_url = format!("http://{}/api/v1", gitea_address());
    format!("{}/{}", api_url, endpoint)
}

pub fn gitea_address() -> &'static str {
    "localhost:3000"
}
