use anyhow::Context;
use octocrab::{Octocrab, OctocrabBuilder};
use secrecy::{ExposeSecret, SecretString};
use tracing::{info, instrument, Span};
use url::Url;

#[derive(Debug)]
pub struct GitHubClient<'a> {
    github: &'a GitHub,
    client: Octocrab,
}

#[derive(Debug)]
pub struct Pr {
    pub branch: String,
    pub title: String,
}

impl<'a> GitHubClient<'a> {
    pub fn new(github: &'a GitHub) -> anyhow::Result<Self> {
        let mut octocrab_builder =
            OctocrabBuilder::new().personal_token(github.token.expose_secret().clone());

        if let Some(base_url) = &github.base_url {
            octocrab_builder = octocrab_builder
                .base_url(base_url.clone())
                .context("Invalid GitHub base url")?;
        }

        let client = octocrab_builder
            .build()
            .context("Failed to build GitHub client")?;

        Ok(Self { github, client })
    }

    pub fn close_other_prs(&self) -> anyhow::Result<()> {
        Ok(())
    }

    #[instrument(
        fields(
            default_branch = tracing::field::Empty,
        )
    )]
    pub async fn open_pr(&self, pr: &Pr) -> anyhow::Result<()> {
        let default_branch = self
            .client
            .repos(&self.github.owner, &self.github.repo)
            .get()
            .await
            .context(format!(
                "failed to retrieve GitHub repository {}/{}",
                self.github.owner, self.github.repo
            ))?
            .default_branch
            .context("failed to retrieve default branch")?;
        Span::current().record("default_branch", &default_branch.as_str());

        let pr = self
            .client
            .pulls(&self.github.owner, &self.github.repo)
            .create(&pr.title, &pr.branch, default_branch)
            .body("release-plz automatic bot")
            .send()
            .await?;

        if let Some(url) = pr.html_url {
            info!("opened pr: {}", url);
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct GitHub {
    pub owner: String,
    pub repo: String,
    pub token: SecretString,
    base_url: Option<Url>,
}

impl GitHub {
    pub fn new(owner: String, repo: String, token: SecretString) -> Self {
        Self {
            owner,
            repo,
            token,
            base_url: None,
        }
    }

    pub fn with_base_url(self, base_url: Url) -> Self {
        Self {
            base_url: Some(base_url),
            ..self
        }
    }
}
