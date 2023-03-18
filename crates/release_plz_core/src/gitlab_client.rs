use anyhow::Context;
use reqwest::header::{HeaderMap, HeaderValue};
use secrecy::{ExposeSecret, SecretString};

use crate::backend::Remote;

#[derive(Debug, Clone)]
pub struct GitLab {
    pub remote: Remote,
}

impl GitLab {
    pub fn new(owner: String, repo: String, token: SecretString) -> Self {
        Self {
            remote: Remote {
                owner,
                repo,
                token,
                base_url: "https://gitlab.com/api/v4".parse().unwrap(),
            },
        }
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
