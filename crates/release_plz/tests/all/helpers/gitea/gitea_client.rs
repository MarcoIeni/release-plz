use crate::helpers::reqwest_utils::ReqwestUtils;

use super::{GiteaContext, GiteaUser};

impl GiteaContext {
    pub fn repo_clone_url(&self) -> String {
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

    fn specific_repo_url(&self, repo_name: &str) -> String {
        super::gitea_endpoint(&format!("repos/{}/{}", self.user.username, repo_name))
    }

    /// Get the repository and return its name.
    async fn get_repo(&self, repo_name: &str) -> String {
        let repo: Repository = self
            .client
            .get(&self.specific_repo_url(repo_name))
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

    pub async fn changed_files_in_pr(&self, pr_number: u64) -> Vec<ChangedFile> {
        let pr_url = format!(
            "http://localhost:3000/api/v1/repos/{}/{}/pulls/{}/files",
            self.user.username, self.repo, pr_number
        );
        self.client
            .get(&pr_url)
            .basic_auth(&self.user.username, Some(&self.user.password))
            .send()
            .await
            .unwrap()
            .ok_if_2xx()
            .await
            .unwrap()
            .json::<Vec<ChangedFile>>()
            .await
            .unwrap()
    }
}

#[derive(Debug, serde::Deserialize)]
pub struct ChangedFile {
    pub filename: String,
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
