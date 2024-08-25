use std::collections::HashMap;

use anyhow::Context as _;
use git_cliff_core::config::ChangelogConfig;
use git_cmd::Repo;

use crate::{diff::Commit, GitClient, RemoteContributor};

#[derive(Debug)]
pub struct RequiredInfo {
    author_name: bool,
    author_email: bool,
    committer_name: bool,
    committer_email: bool,
    remote: bool,
}

pub async fn fill_commit<'a>(
    commit: &'a mut Commit,
    required_info: &RequiredInfo,
    repository: &Repo,
    all_commits: &mut HashMap<String, &'a Commit>,
    git_client: Option<&GitClient>,
) -> anyhow::Result<()> {
    if let Some(existing_commit) = all_commits.get(&commit.id) {
        commit.author = existing_commit.author.clone();
        commit.committer = existing_commit.committer.clone();
    } else {
        if required_info.author_name {
            commit.author.name = Some(repository.get_author_name(&commit.id)?);
        }
        if required_info.author_email {
            commit.author.email = Some(repository.get_author_email(&commit.id)?);
        }
        if required_info.committer_name {
            commit.committer.name = Some(repository.get_committer_name(&commit.id)?);
        }
        if required_info.committer_email {
            commit.committer.email = Some(repository.get_committer_email(&commit.id)?);
        }
        if required_info.remote {
            let remote_commit = git_client
                .context("Remote contributor is required, but no git client is provided")?
                .get_remote_commit(&commit.id)
                .await?;
            commit.remote = RemoteContributor {
                username: remote_commit.username,
            };
        }
        all_commits.insert(commit.id.clone(), commit);
    }
    Ok(())
}

pub fn get_required_info(changelog_config: &ChangelogConfig) -> RequiredInfo {
    let mut required_info = RequiredInfo {
        author_name: false,
        author_email: false,
        committer_name: false,
        committer_email: false,
        remote: false,
    };

    if let Some(body) = changelog_config.body.as_ref() {
        required_info.author_name = body.contains("author.name");
        required_info.author_email = body.contains("author.email");
        required_info.committer_name = body.contains("committer.name");
        required_info.committer_email = body.contains("committer.email");
        required_info.remote = body.contains("commit.remote");
    }

    required_info
}
