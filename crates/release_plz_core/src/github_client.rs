use anyhow::Context;
use reqwest::header::{HeaderMap, HeaderValue};
use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::instrument;
use url::Url;

use crate::pr::Pr;

#[derive(Debug)]
pub struct GitHubClient<'a> {
    github: &'a GitHub,
    client: reqwest::Client,
    base_url: Url,
}

const GITHUB_BASE_URL: &str = "https://api.github.com";

fn default_headers(token: &SecretString) -> anyhow::Result<HeaderMap> {
    let mut headers = HeaderMap::new();
    let header_value: HeaderValue = format!("Bearer {}", token.expose_secret())
        .parse()
        .context("invalid GitHub token")?;
    headers.insert(
        reqwest::header::ACCEPT,
        HeaderValue::from_static("application/vnd.github+json"),
    );
    headers.insert(reqwest::header::AUTHORIZATION, header_value);
    Ok(headers)
}

#[derive(Serialize)]
pub struct CreateReleaseOption<'a> {
    tag_name: &'a str,
    body: &'a str,
    name: &'a str,
}

#[derive(Deserialize)]
struct GitHubPr {
    number: u64,
    html_url: Url,
}

impl<'a> GitHubClient<'a> {
    pub fn new(github: &'a GitHub) -> anyhow::Result<Self> {
        let headers = default_headers(&github.token)?;

        let client = reqwest::Client::builder()
            .user_agent("release-plz")
            .default_headers(headers)
            .build()
            .context("can't build GitHub client")?;

        let base_url = github
            .base_url
            .clone()
            .unwrap_or_else(|| Url::parse(GITHUB_BASE_URL).unwrap());

        Ok(Self {
            github,
            client,
            base_url,
        })
    }

    /// Creates a GitHub release.
    pub async fn create_release(&self, tag: &str, body: &str) -> anyhow::Result<()> {
        let create_release_options = CreateReleaseOption {
            tag_name: tag,
            body,
            name: tag,
        };
        self.client
            .post(format!(
                "{}repos/{}/{}/releases",
                self.base_url, self.github.owner, self.github.repo
            ))
            .json(&create_release_options)
            .send()
            .await
            .context("Failed to create release")?
            .error_for_status()?;
        Ok(())
    }

    pub fn pulls_url(&self) -> String {
        format!(
            "{}repos/{}/{}/pulls",
            self.base_url, self.github.owner, self.github.repo
        )
    }

    /// Close all Prs which branch starts with the given `branch_prefix`.
    pub async fn close_prs_on_branches(&self, branch_prefix: &str) -> anyhow::Result<()> {
        let release_prs: Vec<GitHubPr> = self
            .client
            .get(self.pulls_url())
            .query(&[("state", "open"), ("base", branch_prefix)])
            .send()
            .await
            .context("Failed to retrieve branches")?
            .error_for_status()?
            .json()
            .await
            .context("failed to parse pr")?;

        for pr in release_prs {
            self.close_pr(pr.number).await?;
        }

        Ok(())
    }

    async fn close_pr(&self, pr_number: u64) -> anyhow::Result<()> {
        self.client
            .patch(format!("{}/{}", self.pulls_url(), pr_number))
            .json(&json!({
                "state": "closed",
            }))
            .send()
            .await
            .with_context(|| format!("cannot close pr {pr_number}"))?;
        Ok(())
    }

    #[instrument(skip(self, pr))]
    pub async fn open_pr(&self, pr: &Pr) -> anyhow::Result<Url> {
        let pr: GitHubPr = self
            .client
            .post(self.pulls_url())
            .json(&json!({
                "title": pr.title,
                "body": pr.body,
                "base": pr.base_branch,
                "head": pr.branch
            }))
            .send()
            .await
            .context("Failed to open PR")?
            .error_for_status()?
            .json()
            .await
            .context("Failed to parse PR")?;

        Ok(pr.html_url)
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
