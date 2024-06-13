use anyhow::Context;
use reqwest::header::{HeaderMap, HeaderValue};
use secrecy::{ExposeSecret, SecretString};

use crate::{git::backend::Remote, RepoUrl};

#[derive(Debug, Clone)]
pub struct GitLab {
    pub remote: Remote,
}

impl GitLab {
    pub fn new(url: RepoUrl, token: SecretString) -> anyhow::Result<Self> {
        let base_url = url
            .gitlab_api_url()
            .parse()
            .context("invalid GitLab API URL")?;

        Ok(Self {
            remote: Remote {
                base_url,
                owner: url.owner,
                repo: url.name,
                token,
            },
        })
    }

    pub fn default_headers(&self) -> anyhow::Result<HeaderMap> {
        let mut headers = HeaderMap::new();
        headers.insert(
            reqwest::header::HeaderName::from_static("content-type"),
            reqwest::header::HeaderValue::from_static("application/json"),
        );
        let auth_header: HeaderValue = self
            .remote
            .token
            .expose_secret()
            .parse()
            .context("Invalid Gitlab token")?;
        headers.insert(
            reqwest::header::HeaderName::from_static("private-token"),
            auth_header,
        );
        Ok(headers)
    }
}
