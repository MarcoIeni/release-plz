use crate::backend::Pr;
use secrecy::SecretString;
use tracing::instrument;
use url::Url;

#[derive(Debug)]
pub struct GiteaClient<'a> {
    gitea: &'a Gitea,
}

impl<'a> GiteaClient<'a> {
    pub fn new(gitea: &'a Gitea) -> anyhow::Result<Self> {
        Ok(Self {
            gitea,
        })
    }

    /// Close all Prs which branch starts with the given `branch_prefix`.
    pub async fn close_prs_on_branches(&self, branch_prefix: &str) -> anyhow::Result<()> {
        todo!()
    }

    #[instrument(
    fields(
    default_branch = tracing::field::Empty,
    ),
    skip(pr)
    )]
    pub async fn open_pr(&self, pr: &Pr) -> anyhow::Result<()> {
        todo!()
    }
}

#[derive(Debug)]
pub struct Gitea {
    pub owner: String,
    pub repo: String,
    pub token: SecretString,
    base_url: Url,
}

impl Gitea {
    pub fn new(owner: String, repo: String, token: SecretString, base_url: Url) -> Self {
        Self {
            owner,
            repo,
            token,
            base_url,
        }
    }
}
