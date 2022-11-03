use anyhow::Context;
use octocrab::{models::IssueState, params, Octocrab, OctocrabBuilder};
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
    pub body: String,
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

    /// Close all Prs which branch starts with the given `branch_prefix`.
    pub async fn close_prs_on_branches(&self, branch_prefix: &str) -> anyhow::Result<()> {
        let pulls = self.client.pulls(&self.github.owner, &self.github.repo);

        let mut i: u32 = 1;
        let page_size = 30;
        loop {
            let prs = pulls
                .list()
                .state(params::State::Open)
                .per_page(page_size)
                .page(i)
                .send()
                .await
                .context("Failed to retrieve PRs")?
                .take_items();
            let release_prs = prs
                .iter()
                .filter(|&pr| pr.head.ref_field.starts_with(branch_prefix))
                .map(|pr| pr.number);
            for release_pr in release_prs {
                self.close_pr(release_pr).await?;
            }

            if prs.len() < page_size as usize {
                break;
            }
            i += 1;
        }
        Ok(())
    }

    async fn close_pr(&self, pr_number: u64) -> anyhow::Result<()> {
        self.client
            .issues(&self.github.owner, &self.github.repo)
            .update(pr_number)
            .state(IssueState::Closed)
            .send()
            .await
            .with_context(|| format!("cannot close pr {pr_number}"))?;
        Ok(())
    }

    #[instrument(
        fields(
            default_branch = tracing::field::Empty,
        ),
        skip(pr)
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
        Span::current().record("default_branch", default_branch.as_str());

        let pr = self
            .client
            .pulls(&self.github.owner, &self.github.repo)
            .create(&pr.title, &pr.branch, default_branch)
            .body(&pr.body)
            .send()
            .await
            .context("Failed to open PR")?;

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
