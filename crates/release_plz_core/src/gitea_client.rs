use crate::backend::Pr;
use anyhow::bail;
use reqwest::Method;
use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Serialize};
use tracing::{debug, instrument};
use url::Url;

#[derive(Debug)]
pub struct GiteaClient<'a> {
    gitea: &'a Gitea,
    client: reqwest::Client,
}

impl<'a> GiteaClient<'a> {
    pub fn new(gitea: &'a Gitea) -> anyhow::Result<Self> {
        let client = reqwest::Client::builder().use_rustls_tls().build()?;
        Ok(Self { gitea, client })
    }

    /// Close all Prs which branch starts with the given `branch_prefix`.
    pub async fn close_prs_on_branches(&self, branch_prefix: &str) -> anyhow::Result<()> {
        let mut page = 1;
        let page_size = 30;
        loop {
            let req = self
                .client
                .request(
                    Method::GET,
                    format!(
                        "{}/repos/{}/{}/pulls?token={}&state=open&page={}&limit={}",
                        self.gitea.api_url,
                        self.gitea.owner,
                        self.gitea.repo,
                        self.gitea.token.expose_secret(),
                        page,
                        page_size
                    ),
                )
                .header("accept", "application/json")
                .build()?;
            debug!(
                "Loading prs from {}/{}, page {page}",
                self.gitea.owner, self.gitea.repo
            );
            let prs: Vec<RepoPr> = self
                .client
                .execute(req)
                .await?
                .error_for_status()?
                .json()
                .await?;

            for pr in &prs {
                if pr.head.ref_field.starts_with(branch_prefix) {
                    debug!(
                        "Closing pr #{} in {}/{}",
                        pr.id, self.gitea.owner, self.gitea.repo
                    );
                    let req = self
                        .client
                        .request(
                            Method::PATCH,
                            format!(
                                "{}/repos/{}/{}/pulls/{}?token={}",
                                self.gitea.api_url,
                                self.gitea.owner,
                                self.gitea.repo,
                                &pr.id,
                                self.gitea.token.expose_secret(),
                            ),
                        )
                        .json(&EditPullRequest { state: "closed" })
                        .build()?;
                    self.client.execute(req).await?.error_for_status()?;
                }
            }

            if prs.len() < page_size as usize {
                break;
            }
            page += 1;
        }

        Ok(())
    }

    #[instrument(
    fields(
    default_branch = tracing::field::Empty,
    ),
    skip(pr)
    )]
    pub async fn open_pr(&self, pr: &Pr) -> anyhow::Result<()> {
        let req_body = OpenPrBody {
            title: &pr.title,
            body: &pr.body,
            base: &pr.base_branch,
            head: &pr.branch,
        };

        let req = self
            .client
            .request(
                Method::POST,
                format!(
                    "{}/repos/{}/{}/pulls?token={}",
                    self.gitea.api_url,
                    self.gitea.owner,
                    self.gitea.repo,
                    self.gitea.token.expose_secret()
                ),
            )
            .json(&req_body)
            .build()?;
        debug!(
            "Opening PR in {}/{}: {:?}",
            self.gitea.owner, self.gitea.repo, req
        );
        self.client.execute(req).await?.error_for_status()?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct Gitea {
    pub owner: String,
    pub repo: String,
    pub token: SecretString,
    api_url: Url,
}

impl Gitea {
    pub fn new(
        owner: String,
        repo: String,
        token: SecretString,
        base_url: Url,
    ) -> anyhow::Result<Self> {
        match base_url.scheme() {
            "http" | "https" => {}
            _ => bail!(
                "invalid scheme for gitea url, only `http` and `https` are supported: {base_url}"
            ),
        }
        let api_url = base_url
            .as_str()
            .strip_suffix('/')
            .unwrap_or_else(|| base_url.as_str());
        let api_url = api_url.strip_suffix(".git").unwrap_or(api_url);
        let api_url = api_url
            .strip_suffix(&format!("/{owner}/{repo}"))
            .unwrap_or(api_url);
        Ok(Self {
            owner,
            repo,
            token,
            api_url: Url::parse(&format!("{api_url}/api/v1"))?,
        })
    }
}

#[derive(Serialize, Deserialize)]
struct OpenPrBody<'a> {
    title: &'a str,
    body: &'a str,
    base: &'a str,
    head: &'a str,
}

#[derive(Serialize, Deserialize)]
struct RepoPr {
    pub id: u64,
    pub head: Commit,
}

#[derive(Serialize, Deserialize)]
struct Commit {
    #[serde(rename = "ref")]
    pub ref_field: String,
}

#[derive(Serialize, Deserialize)]
struct EditPullRequest {
    state: &'static str,
}
