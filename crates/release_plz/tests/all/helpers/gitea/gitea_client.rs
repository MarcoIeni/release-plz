use crate::helpers::reqwest_utils::ReqwestUtils;

use super::{GiteaContext, GiteaUser};

impl GiteaContext {
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

        let repo: Repository = self.client
            .get(repo_url)
            .basic_auth(&self.user.username, Some(&self.user.password))
            .send()
            .await
            .unwrap()
            .ok_if_2xx()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();

        repo.name
    }

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
