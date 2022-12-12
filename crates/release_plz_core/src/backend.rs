use crate::gitea_client::{Gitea, GiteaClient};
use crate::github_client::GitHubClient;
use crate::GitHub;
use clap::ValueEnum;
use enum_kinds::EnumKind;
use std::fmt::{Display, Formatter};
use tracing::instrument;

#[derive(EnumKind, Debug)]
#[enum_kind(GitBackendKind, derive(ValueEnum))]
pub enum GitBackend {
    Github(GitHub),
    Gitea(Gitea),
}

impl Display for GitBackendKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            GitBackendKind::Github => f.write_str("github"),
            GitBackendKind::Gitea => f.write_str("gitea"),
        }
    }
}

#[derive(Debug)]
pub enum GitClient<'a> {
    GitHub(GitHubClient<'a>),
    Gitea(GiteaClient<'a>),
}

impl<'a> GitClient<'a> {
    pub fn new(backend: &'a GitBackend) -> anyhow::Result<Self> {
        Ok(match backend {
            GitBackend::Github(g) => GitClient::GitHub(GitHubClient::new(g)?),
            GitBackend::Gitea(g) => GitClient::Gitea(GiteaClient::new(g)?),
        })
    }

    /// Close all Prs which branch starts with the given `branch_prefix`.
    pub async fn close_prs_on_branches(&self, branch_prefix: &str) -> anyhow::Result<()> {
        match self {
            GitClient::GitHub(g) => g.close_prs_on_branches(branch_prefix).await,
            GitClient::Gitea(g) => g.close_prs_on_branches(branch_prefix).await,
        }
    }

    #[instrument(
    fields(
    default_branch = tracing::field::Empty,
    ),
    skip(pr)
    )]
    pub async fn open_pr(&self, pr: &Pr) -> anyhow::Result<()> {
        match self {
            GitClient::GitHub(g) => g.open_pr(pr).await,
            GitClient::Gitea(g) => g.open_pr(pr).await,
        }
    }
}

#[derive(Debug)]
pub struct Pr {
    pub branch: String,
    pub title: String,
    pub body: String,
}
