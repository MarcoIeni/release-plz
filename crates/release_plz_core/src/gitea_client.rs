use crate::backend::Pr;
use anyhow::bail;
use reqwest::header::HeaderValue;
use reqwest::Method;
use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Serialize};
use tracing::instrument;
use url::Url;

#[derive(Debug)]
pub struct GiteaClient<'a> {
    gitea: &'a Gitea,
    client: reqwest::Client,
    token: HeaderValue,
}

impl<'a> GiteaClient<'a> {
    pub fn new(gitea: &'a Gitea) -> anyhow::Result<Self> {
        let client = reqwest::Client::builder().use_rustls_tls().build()?;
        let token = get_token(gitea)?;
        Ok(Self {
            gitea,
            client,
            token,
        })
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
                        "{}api/v1/repos/{}/{}/pulls?state=open&page={}&limit={}",
                        self.gitea.base_url, self.gitea.owner, self.gitea.repo, page, page_size
                    ),
                )
                .header("Authorization", self.token.clone())
                .build()?;
            let prs: Vec<RepoPr> = self
                .client
                .execute(req)
                .await?
                .error_for_status()?
                .json()
                .await?;

            for pr in &prs {
                if pr.head.ref_field.starts_with(branch_prefix) {
                    let req = self
                        .client
                        .request(
                            Method::PATCH,
                            format!(
                                "{}api/v1/repos/{}/{}/pulls/{}",
                                self.gitea.base_url, self.gitea.owner, self.gitea.repo, &pr.id,
                            ),
                        )
                        .header("Authorization", self.token.clone())
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
            base: &pr.branch,
        };

        let req = self
            .client
            .request(
                Method::POST,
                format!(
                    "{}api/v1/repos/{}/{}/pulls",
                    self.gitea.base_url, self.gitea.owner, self.gitea.repo
                ),
            )
            .header("Authorization", self.token.clone())
            .json(&req_body)
            .build()?;
        self.client.execute(req).await?.error_for_status()?;
        Ok(())
    }
}

fn get_token(gitea: &Gitea) -> anyhow::Result<HeaderValue> {
    let mut token = HeaderValue::from_str(&format!("token {}", gitea.token.expose_secret()))?;
    token.set_sensitive(true);
    Ok(token)
}

#[derive(Debug)]
pub struct Gitea {
    pub owner: String,
    pub repo: String,
    pub token: SecretString,
    base_url: Url,
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
        Ok(Self {
            owner,
            repo,
            token,
            base_url,
        })
    }
}

#[derive(Serialize, Deserialize)]
struct OpenPrBody<'a> {
    title: &'a str,
    body: &'a str,
    base: &'a str,
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
