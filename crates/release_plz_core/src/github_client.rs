use anyhow::Context;
use reqwest::header::{HeaderMap, HeaderValue};
use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::{debug, instrument};
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
pub struct GitHubPr {
    pub number: u64,
    html_url: Url,
    pub head: Commit,
}

impl GitHubPr {
    pub fn branch(&self) -> &str {
        self.head.ref_field.as_str()
    }
}

#[derive(Deserialize)]
pub struct Commit {
    #[serde(rename = "ref")]
    pub ref_field: String,
    pub sha: String,
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

    pub async fn opened_prs(&self, branch_prefix: &str) -> anyhow::Result<Vec<GitHubPr>> {
        let mut page = 1;
        let page_size = 30;
        let mut release_prs: Vec<GitHubPr> = vec![];
        loop {
            let prs: Vec<GitHubPr> = self
                .client
                .get(self.pulls_url())
                .query(&[("state", "open")])
                .query(&[("page", page)])
                .query(&[("per_page", page_size)])
                .send()
                .await
                .context("Failed to retrieve branches")?
                .error_for_status()?
                .json()
                .await
                .context("failed to parse pr")?;
            let prs_len = prs.len();
            let current_release_prs: Vec<GitHubPr> = prs
                .into_iter()
                .filter(|pr| pr.head.ref_field.starts_with(branch_prefix))
                .collect();
            release_prs.extend(current_release_prs);
            if prs_len < page_size as usize {
                break;
            }
            page += 1;
        }
        Ok(release_prs)
    }

    /// Close all Prs which branch starts with the given `branch_prefix`.
    pub async fn close_prs_on_branches(&self, branch_prefix: &str) -> anyhow::Result<()> {
        for pr in self.opened_prs(branch_prefix).await? {
            debug!("closing pr {}", pr.number);
            self.close_pr(pr.number).await?;
        }
        Ok(())
    }

    pub async fn close_pr(&self, pr_number: u64) -> anyhow::Result<()> {
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
}

/// Returns the list of contributors for the given commits,
/// excluding the PR author and bots.
pub fn contributors_from_commits(commits: &[PrCommit]) -> Vec<String> {
    let mut contributors = commits
        .into_iter()
        .skip(1) // skip pr author
        .flat_map(|commit| &commit.author)
        .filter(|author| !author.login.ends_with("[bot]")) // ignore bots
        .map(|author| author.login.clone())
        .collect::<Vec<_>>();
    contributors.dedup();
    contributors
}

#[derive(Deserialize)]
pub struct PrCommit {
    pub author: Option<Author>,
    parents: Vec<CommitParent>,
}

impl PrCommit {
    /// Get the parent commit sha.
    pub fn parent(&self) -> Option<&str> {
        self.parents.get(0).map(|c| c.sha.as_str())
    }
}

#[derive(Deserialize)]
pub struct CommitParent {
    pub sha: String,
}

#[derive(Deserialize)]
pub struct Author {
    login: String,
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
                parents: vec![CommitParent {
                    sha: "123".to_string(),
                }],
            },
            PrCommit {
                author: Some(Author {
                    login: "marco".to_string(),
                }),
                parents: vec![CommitParent {
                    sha: "123".to_string(),
                }],
            },
            PrCommit {
                author: Some(Author {
                    login: "release[bot]".to_string(),
                }),
                parents: vec![],
            },
            PrCommit {
                author: None,
                parents: vec![],
            },
        ];
        let contributors = contributors_from_commits(&commits);
        assert_eq!(contributors, vec!["marco"]);
    }
}
