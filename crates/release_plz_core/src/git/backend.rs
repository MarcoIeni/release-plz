use crate::git::{gitea_client::Gitea, gitlab_client::GitLab};
use crate::{GitHub, GitReleaseInfo};

use crate::pr::Pr;
use anyhow::Context;
use reqwest::header::HeaderMap;
use reqwest::Url;
use reqwest_middleware::ClientBuilder;
use reqwest_retry::{policies::ExponentialBackoff, RetryTransientMiddleware};
use secrecy::SecretString;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::{debug, info, instrument, warn};

#[derive(Debug, Clone)]
pub enum GitBackend {
    Github(GitHub),
    Gitea(Gitea),
    Gitlab(GitLab),
}

impl GitBackend {
    fn default_headers(&self) -> anyhow::Result<HeaderMap> {
        match self {
            GitBackend::Github(g) => g.default_headers(),
            GitBackend::Gitea(g) => g.default_headers(),
            GitBackend::Gitlab(g) => g.default_headers(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BackendType {
    Github,
    Gitea,
    Gitlab,
}

#[derive(Debug)]
pub struct GitClient {
    pub backend: BackendType,
    pub remote: Remote,
    pub client: reqwest_middleware::ClientWithMiddleware,
}

#[derive(Debug, Clone)]
pub struct Remote {
    pub owner: String,
    pub repo: String,
    pub token: SecretString,
    pub base_url: Url,
}

impl Remote {
    pub fn owner_slash_repo(&self) -> String {
        format!("{}/{}", self.owner, self.repo)
    }
}

#[derive(Deserialize)]
pub struct PrCommit {
    pub author: Option<Author>,
}

#[derive(Deserialize)]
pub struct CommitParent {
    pub sha: String,
}

#[derive(Deserialize)]
pub struct Author {
    login: String,
}

#[derive(Serialize)]
pub struct CreateReleaseOption<'a> {
    tag_name: &'a str,
    body: &'a str,
    name: &'a str,
    draft: &'a bool,
    prerelease: &'a bool,
}

#[derive(Deserialize, Clone, Debug)]
pub struct GitPr {
    pub number: u64,
    pub html_url: Url,
    pub head: Commit,
    pub title: String,
    pub body: Option<String>,
}

impl GitPr {
    pub fn branch(&self) -> &str {
        self.head.ref_field.as_str()
    }
}

#[derive(Deserialize, Clone, Debug)]
pub struct Commit {
    #[serde(rename = "ref")]
    pub ref_field: String,
    pub sha: String,
}

#[derive(Serialize, Default)]
pub struct PrEdit {
    #[serde(skip_serializing_if = "Option::is_none")]
    title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    body: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    state: Option<String>,
}

impl PrEdit {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn with_body(mut self, body: impl Into<String>) -> Self {
        self.body = Some(body.into());
        self
    }

    pub fn with_state(mut self, state: impl Into<String>) -> Self {
        self.state = Some(state.into());
        self
    }

    pub fn contains_edit(&self) -> bool {
        self.title.is_some() || self.body.is_some() || self.state.is_some()
    }
}

impl GitClient {
    pub fn new(backend: GitBackend) -> anyhow::Result<Self> {
        let client = {
            let headers = backend.default_headers()?;
            let reqwest_client = reqwest::Client::builder()
                .user_agent("release-plz")
                .default_headers(headers)
                .build()
                .context("can't build Git client")?;

            let retry_policy = ExponentialBackoff::builder().build_with_max_retries(3);
            ClientBuilder::new(reqwest_client)
                // Retry failed requests.
                .with(RetryTransientMiddleware::new_with_policy(retry_policy))
                .build()
        };

        let (backend, remote) = match backend {
            GitBackend::Github(g) => (BackendType::Github, g.remote),
            GitBackend::Gitea(g) => (BackendType::Gitea, g.remote),
            GitBackend::Gitlab(g) => (BackendType::Gitlab, g.remote),
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
            BackendType::Gitlab => {
                unimplemented!("Gitlab support for `release-plz release-pr is not implemented yet")
            }
        }
    }

    /// Creates a GitHub/Gitea release.
    pub async fn create_release(&self, release_info: &GitReleaseInfo) -> anyhow::Result<()> {
        match self.backend {
            BackendType::Github | BackendType::Gitea => {
                self.create_github_release(release_info).await
            }
            BackendType::Gitlab => self.create_gitlab_release(release_info).await,
        }
        .context("Failed to create release")
    }

    /// Same as Gitea.
    pub async fn create_github_release(&self, release_info: &GitReleaseInfo) -> anyhow::Result<()> {
        let create_release_options = CreateReleaseOption {
            tag_name: &release_info.git_tag,
            body: &release_info.release_body,
            name: &release_info.release_name,
            draft: &release_info.draft,
            prerelease: &release_info.pre_release,
        };
        self.client
            .post(format!("{}/releases", self.repo_url()))
            .json(&create_release_options)
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }

    pub async fn create_gitlab_release(&self, release_info: &GitReleaseInfo) -> anyhow::Result<()> {
        #[derive(Serialize)]
        pub struct GitlabReleaseOption<'a> {
            name: &'a str,
            tag_name: &'a str,
            description: &'a str,
        }
        let gitlab_release_options = GitlabReleaseOption {
            name: &release_info.release_name,
            tag_name: &release_info.git_tag,
            description: &release_info.release_body,
        };
        self.client
            .post(format!(
                "{}/projects/{}%2F{}/releases",
                self.remote.base_url, self.remote.owner, self.remote.repo
            ))
            .json(&gitlab_release_options)
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }

    pub fn pulls_url(&self) -> String {
        format!("{}/pulls", self.repo_url())
    }

    pub fn issues_url(&self) -> String {
        format!("{}/issues", self.repo_url())
    }

    fn repo_url(&self) -> String {
        format!(
            "{}repos/{}",
            self.remote.base_url,
            self.remote.owner_slash_repo()
        )
    }

    /// Get all opened Prs which branch starts with the given `branch_prefix`.
    pub async fn opened_prs(&self, branch_prefix: &str) -> anyhow::Result<Vec<GitPr>> {
        let mut page = 1;
        let page_size = 30;
        let mut release_prs: Vec<GitPr> = vec![];
        loop {
            debug!(
                "Loading prs from {}, page {page}",
                self.remote.owner_slash_repo()
            );
            let prs: Vec<GitPr> = self
                .opened_prs_page(page, page_size)
                .await
                .context("Failed to retrieve open PRs")?;
            let prs_len = prs.len();
            let current_release_prs: Vec<GitPr> = prs
                .into_iter()
                .filter(|pr| pr.head.ref_field.starts_with(branch_prefix))
                .collect();
            release_prs.extend(current_release_prs);
            if prs_len < page_size {
                break;
            }
            page += 1;
        }
        Ok(release_prs)
    }

    async fn opened_prs_page(&self, page: i32, page_size: usize) -> anyhow::Result<Vec<GitPr>> {
        self.client
            .get(self.pulls_url())
            .query(&[("state", "open")])
            .query(&[("page", page)])
            .query(&[(self.per_page(), page_size)])
            .send()
            .await?
            .error_for_status()?
            .json()
            .await
            .context("failed to parse pr")
    }

    #[instrument(skip(self))]
    pub async fn close_pr(&self, pr_number: u64) -> anyhow::Result<()> {
        debug!("closing pr #{pr_number}");
        let edit = PrEdit::new().with_state("closed");
        self.edit_pr(pr_number, &edit)
            .await
            .with_context(|| format!("cannot close pr {pr_number}"))?;
        info!("closed pr #{pr_number}");
        Ok(())
    }

    pub async fn edit_pr(&self, pr_number: u64, pr_edit: &PrEdit) -> anyhow::Result<()> {
        debug!("editing pr");
        self.client
            .patch(format!("{}/{}", self.pulls_url(), pr_number))
            .json(pr_edit)
            .send()
            .await
            .with_context(|| format!("cannot edit pr {pr_number}"))?;
        Ok(())
    }

    #[instrument(skip(self, pr))]
    pub async fn open_pr(&self, pr: &Pr) -> anyhow::Result<GitPr> {
        debug!("Opening PR in {}", self.remote.owner_slash_repo());
        let git_pr: GitPr = self
            .client
            .post(self.pulls_url())
            .json(&json!({
                "title": pr.title,
                "body": pr.body,
                "base": pr.base_branch,
                "head": pr.branch,
                "draft": pr.draft,
            }))
            .send()
            .await?
            .error_for_status()?
            .json()
            .await
            .context("Failed to parse PR")?;

        info!("opened pr: {}", git_pr.html_url);
        self.add_labels(pr, git_pr.number)
            .await
            .context("Failed to add labels")?;
        Ok(git_pr)
    }

    #[instrument(skip(self, pr))]
    async fn add_labels(&self, pr: &Pr, pr_number: u64) -> anyhow::Result<()> {
        if pr.labels.is_empty() {
            return Ok(());
        }
        if self.backend != BackendType::Github {
            warn!("PR labels are only supported on Github");
            return Ok(());
        }
        self.client
            .post(format!("{}/{}/labels", self.issues_url(), pr_number))
            .json(&json!({
                "labels": pr.labels
            }))
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }

    pub async fn pr_commits(&self, pr_number: u64) -> anyhow::Result<Vec<PrCommit>> {
        self.client
            .get(format!("{}/{}/commits", self.pulls_url(), pr_number))
            .send()
            .await?
            .error_for_status()?
            .json()
            .await
            .context("can't parse commits")
    }

    pub async fn associated_prs(&self, commit: &str) -> anyhow::Result<Vec<GitPr>> {
        let url = match self.backend {
            BackendType::Github => {
                format!("{}/commits/{}/pulls", self.repo_url(), commit)
            }
            BackendType::Gitea => {
                format!("{}/commits/{}/pull", self.repo_url(), commit)
            }
            BackendType::Gitlab => {
                unimplemented!("Gitlab support for `release-plz release-pr is not implemented yet")
            }
        };

        let response = self.client.get(url).send().await?;
        if response.status() == 404 {
            debug!("No associated PRs for commit {commit}");
            return Ok(vec![]);
        }
        debug!("Associated PR found. Status: {}", response.status());
        let response = response.error_for_status()?;

        let prs = match self.backend {
            BackendType::Github => {
                let prs: Vec<GitPr> = response
                    .json()
                    .await
                    .context("can't parse associated PRs")?;
                prs
            }
            BackendType::Gitea => {
                let pr: GitPr = response.json().await.context("can't parse associated PR")?;
                vec![pr]
            }
            BackendType::Gitlab => {
                unimplemented!("Gitlab support for `release-plz release-pr is not implemented yet")
            }
        };

        let prs_numbers = prs.iter().map(|pr| pr.number).collect::<Vec<_>>();
        debug!("Associated PRs for commit {commit}: {:?}", prs_numbers);
        Ok(prs)
    }
}

/// Returns the list of contributors for the given commits,
/// excluding the PR author and bots.
pub fn contributors_from_commits(commits: &[PrCommit]) -> Vec<String> {
    let mut contributors = commits
        .iter()
        .skip(1) // skip pr author
        .flat_map(|commit| &commit.author)
        .filter(|author| !author.login.ends_with("[bot]")) // ignore bots
        .map(|author| author.login.clone())
        .collect::<Vec<_>>();
    contributors.dedup();
    contributors
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn contributors_are_extracted_from_commits() {
        let commits = vec![
            PrCommit {
                author: Some(Author {
                    login: "bob".to_string(),
                }),
            },
            PrCommit {
                author: Some(Author {
                    login: "marco".to_string(),
                }),
            },
            PrCommit {
                author: Some(Author {
                    login: "release[bot]".to_string(),
                }),
            },
            PrCommit { author: None },
        ];
        let contributors = contributors_from_commits(&commits);
        assert_eq!(contributors, vec!["marco"]);
    }
}
