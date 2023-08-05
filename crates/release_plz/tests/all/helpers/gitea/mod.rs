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
    repo: String,
    client: reqwest::Client,
}
