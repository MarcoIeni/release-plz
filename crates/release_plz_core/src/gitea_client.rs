use crate::RepoUrl;
use anyhow::{bail, Context};
use reqwest::header::HeaderMap;
use reqwest::header::HeaderValue;
use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct Gitea {
    pub owner: String,
    pub repo: String,
    pub token: SecretString,
    pub api_url: String,
}

impl Gitea {
    pub fn new(url: RepoUrl, token: SecretString) -> anyhow::Result<Self> {
        match url.scheme.as_str() {
            "http" | "https" => {}
            _ => bail!(
                "invalid scheme for gitea url, only `http` and `https` are supported: {url:?}"
            ),
        }

        Ok(Self {
            api_url: url.gitea_api_url(),
            owner: url.owner,
            repo: url.name,
            token,
        })
    }

    pub fn default_headers(&self) -> anyhow::Result<HeaderMap> {
        let mut headers = HeaderMap::new();
        let auth_header: HeaderValue = format!("token {}", self.token.expose_secret())
            .parse()
            .context("invalid Gitea token")?;
        headers.insert(reqwest::header::AUTHORIZATION, auth_header);
        Ok(headers)
    }
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
