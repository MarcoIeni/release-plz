use crate::gitea_client::Gitea;
use crate::GitHub;

use anyhow::Context;
use reqwest::Url;
use reqwest::header::HeaderMap;
use secrecy::SecretString;

#[derive(Debug, Clone)]
pub enum GitBackend {
    Github(GitHub),
    Gitea(Gitea),
}

impl GitBackend {
    fn default_headers(&self) -> anyhow::Result<HeaderMap> {
        match self {
            GitBackend::Github(g) => g.default_headers(),
            GitBackend::Gitea(g) => g.default_headers(),
        }
    }
}

#[derive(Debug)]
pub enum BackendType {
    Github,
    Gitea,
}

#[derive(Debug)]
pub struct GitClient {
    backend: BackendType,
    pub remote: Remote,
    pub client: reqwest::Client,
}

#[derive(Debug)]
pub struct Remote {
    pub owner: String,
    pub repo: String,
    pub token: SecretString,
    pub base_url: Url,
}

impl GitClient {
    pub fn new(backend: GitBackend) -> anyhow::Result<Self> {
        let headers = backend.default_headers()?;

        let client = reqwest::Client::builder()
            .user_agent("release-plz")
            .default_headers(headers)
            .build()
            .context("can't build GitHub client")?;

        let (backend, remote) = match backend {
            GitBackend::Github(g) => (
                BackendType::Github,
                Remote {
                    owner: g.owner,
                    repo: g.repo,
                    token: g.token,
                    base_url: g.base_url,
                },
            ),
            GitBackend::Gitea(g) => (
                BackendType::Gitea,
                Remote {
                    owner: g.owner,
                    repo: g.repo,
                    token: g.token,
                    base_url: g.api_url,
                },
            ),
        };
        Ok(Self {
            remote,
            backend,
            client,
        })
    }

    pub fn per_page(&self) -> &str {
        match self.backend {
            BackendType::Github => "per_page",
            BackendType::Gitea => "limit",
        }
    }
}
