use std::{collections::HashMap, fs, path::Path};

use anyhow::Result;
use base64::prelude::*;
use git_cmd::Repo;
use serde_json::Value;
use tracing::{debug, trace};

use crate::GitClient;

const API_ENDPOINT: &str = "https://api.github.com/graphql";

/// TODO: add tests
/// Commit all the changes (except typestates) that are present in the repository
/// using GitHub's [GraphQL api](https://docs.github.com/en/graphql).
pub async fn commit_changes(client: &GitClient, repo: &Repo, message: &str) -> Result<()> {
    let owner_and_repo = format!("{}/{}", client.remote.owner, client.remote.repo);
    let branch = repo.current_branch()?;
    let current_head = repo.current_head()?;
    let deletions = removed_files(repo)?;
    let changes = changed_files(repo)?;

    let commit_query = format_commit_query(
        &owner_and_repo,
        &branch,
        message,
        &current_head,
        &deletions,
        &changes,
        repo.directory(),
    )?;

    debug!("Sending createCommitOnBranch to {}", API_ENDPOINT);
    trace!("{}", commit_query);

    let mut json = HashMap::new();
    json.insert("query", commit_query.as_str());

    let res: Value = client
        .client
        .post(API_ENDPOINT)
        .json(&json)
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

// get the list of changes in repository excluding typechanges and removed files
fn changed_files(repo: &Repo) -> Result<Vec<String>> {
    repo.changes(|line| !line.starts_with("T ") && !line.starts_with("D "))
}

// get the list of removed files in repository
fn removed_files(repo: &Repo) -> Result<Vec<String>> {
    repo.changes(|line| line.starts_with("D "))
}

// format a graphql query to create commit on branch
fn format_commit_query(
    owner_and_repo: &str,
    branch: &str,
    message: &str,
    current_head: &str,
    deletions: &[impl AsRef<Path>],
    additions: &[impl AsRef<Path>],
    repo_dir: impl AsRef<Path>,
) -> Result<String> {
    let deletions = format_deletions(deletions)?;
    let additions = format_additions(repo_dir, additions)?;
    Ok(format!(
        r#"mutation {{
          createCommitOnBranch(input: {{
            branch: {{
              repositoryNameWithOwner: "{owner_and_repo}",
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
fn format_additions(repo_dir: impl AsRef<Path>, paths: &[impl AsRef<Path>]) -> Result<String> {
    let repo_dir = repo_dir.as_ref();
    let mut additions = String::new();
    let mut has_previous = false;

    additions.push('[');

    for path in paths {
        if has_previous {
            additions.push_str(",\n");
        }
        additions.push_str(&format_addition(repo_dir, path)?);

        has_previous = true;
    }

    additions.push(']');

    Ok(additions)
}

// format a single object containing changed or added file
fn format_addition(repo_dir: impl AsRef<Path>, path: impl AsRef<Path>) -> Result<String> {
    let path = path.as_ref();
    let realpath = repo_dir.as_ref().join(path);

    // TODO: async
    let content = BASE64_STANDARD.encode(fs::read(realpath)?);

    Ok(format!(
        r#"{{ path: "{}", contents: "{content}" }}"#,
        path.display()
    ))
}
