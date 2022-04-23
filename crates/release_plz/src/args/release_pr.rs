use anyhow::anyhow;
use release_plz_core::GitHub;
use secrecy::SecretString;
use url::Url;

use super::update::Update;

#[derive(clap::Parser, Debug)]
pub struct ReleasePr {
    #[clap(flatten)]
    pub update: Update,
    /// GitHub token used to create the pull request.
    #[clap(long, forbid_empty_values(true))]
    pub github_token: SecretString,
    /// GitHub repository url where your project is hosted.
    #[clap(long, forbid_empty_values(true))]
    pub repo_url: Url,
}

impl ReleasePr {
    pub fn github(&self) -> anyhow::Result<GitHub> {
        let segments = self
            .repo_url
            .path_segments()
            .map(|c| c.collect::<Vec<_>>())
            .ok_or_else(|| {
                anyhow!(
                    "cannot find github owner and repo from url {}",
                    self.repo_url
                )
            })?;
        let owner = segments
            .get(0)
            .ok_or_else(|| anyhow!("cannot find github owner from url {}", self.repo_url))?
            .to_string();
        let repo = segments
            .get(1)
            .ok_or_else(|| anyhow!("cannot find github repo from url {}", self.repo_url))?
            .to_string();
        Ok(GitHub::new(owner, repo, self.github_token.clone()))
    }
}
