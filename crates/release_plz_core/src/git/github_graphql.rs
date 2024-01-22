use std::path::PathBuf;
use std::{collections::HashMap, path::Path};

use anyhow::Result;
use base64::prelude::*;
use git_cmd::Repo;
use serde_json::Value;
use tokio::fs;
use tracing::{debug, trace};
use url::Url;

use crate::git::backend::Remote;
use crate::GitClient;

/// Commit all the changes (except typestates) that are present in the repository
/// using GitHub's [GraphQL api](https://docs.github.com/en/graphql/reference/mutations#createcommitonbranch).
pub async fn commit_changes(
    client: &GitClient,
    repo: &Repo,
    message: &str,
    branch: &str,
) -> Result<()> {
    let commit = GithubCommit::new(&client.remote.owner_slash_repo(), repo, message, branch)?;
    let graphql_endpoint = get_graphql_endpoint(&client.remote);

    let commit_query = commit.format_query().await?;
    debug!("Sending createCommitOnBranch to {}", graphql_endpoint);
    trace!("{}", commit_query);

    let json_body: HashMap<&str, &str> = [("query", commit_query.as_str())].into();

    let res: Value = client
        .client
        .post(graphql_endpoint)
        .json(&json_body)
        .send()
        .await?
        .json()
        .await?;

    if let Some(errors) = res.get("errors").and_then(Value::as_array) {
        anyhow::bail!(
            "createCommitOnBranch returned errors: {:?}",
            serde_json::to_string(errors)?
        );
    }

    Ok(())
}

fn get_graphql_endpoint(remote: &Remote) -> Url {
    let mut base_url = remote.base_url.clone();
    base_url.set_path("graphql");

    base_url
}

// get the list of changes in repository excluding typechanges and removed files
fn changed_files(repo: &Repo) -> Result<Vec<String>> {
    repo.changes(|line| !line.starts_with("T ") && !line.starts_with("D "))
}

// get the list of removed files in repository
fn removed_files(repo: &Repo) -> Result<Vec<String>> {
    repo.changes(|line| line.starts_with("D "))
}

struct GithubCommit {
    owner_slash_repo: String,
    branch: String,
    message: String,
    current_head: String,
    deletions: Vec<String>,
    additions: Vec<String>,
    repo_dir: PathBuf,
}

impl GithubCommit {
    fn new(owner_slash_repo: &str, repo: &Repo, message: &str, branch: &str) -> Result<Self> {
        Ok(Self {
            owner_slash_repo: owner_slash_repo.to_owned(),
            branch: branch.to_owned(),
            message: message.to_owned(),
            current_head: repo.current_commit_hash()?,
            deletions: removed_files(repo)?,
            additions: changed_files(repo)?,
            repo_dir: repo.directory().to_owned(),
        })
    }

    // format a graphql query to create commit on branch
    async fn format_query(&self) -> Result<String> {
        let GithubCommit {
            owner_slash_repo,
            branch,
            message,
            current_head,
            ..
        } = self;
        let deletions = format_deletions(&self.deletions)?;
        let additions = format_additions(&self.repo_dir, &self.additions).await?;
        Ok(format!(
            r#"mutation {{
              createCommitOnBranch(input: {{
                branch: {{
                  repositoryNameWithOwner: "{owner_slash_repo}",
                  branchName: "{branch}"
                }},
                message: {{ headline: "{message}" }},
                expectedHeadOid: "{current_head}",
                fileChanges: {{
                  deletions: {deletions},
                  additions: {additions}
                }}
              }}) {{
                commit {{
                  author {{
                    name,
                    email
                  }}
                }}
              }}
            }}"#
        ))
    }
}

// format a list of deleted files for a commit query
fn format_deletions(paths: &[impl AsRef<Path>]) -> Result<String> {
    let mut deletions = String::new();
    let mut has_previous = false;

    deletions.push('[');

    for path in paths {
        if has_previous {
            deletions.push_str(",\n");
        }
        deletions.push_str(&format!(r#"{{ path: "{}" }}"#, path.as_ref().display()));

        has_previous = true;
    }

    deletions.push(']');

    Ok(deletions)
}

// format a list of modified/added files for a commit query
async fn format_additions(
    repo_dir: impl AsRef<Path>,
    paths: &[impl AsRef<Path>],
) -> Result<String> {
    let repo_dir = repo_dir.as_ref();
    let mut additions = String::new();
    let mut has_previous = false;

    additions.push('[');

    for path in paths {
        if has_previous {
            additions.push_str(",\n");
        }

        let realpath = repo_dir.join(path);
        let content = BASE64_STANDARD.encode(fs::read(realpath).await?);

        additions.push_str(&format!(
            r#"{{ path: "{}", contents: "{content}" }}"#,
            path.as_ref().display()
        ));

        has_previous = true;
    }

    additions.push(']');

    Ok(additions)
}
