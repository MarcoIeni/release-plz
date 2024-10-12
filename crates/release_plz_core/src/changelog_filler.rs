use std::collections::HashMap;

use anyhow::Context as _;
use git_cliff_core::{config::ChangelogConfig, contributor::RemoteContributor};
use git_cmd::Repo;

use crate::{diff::Commit, GitClient};

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
        commit.remote = existing_commit.remote.clone();
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
                .context("The changelog template requires information from the remote, but git token wasn't provided")?
                .get_remote_commit(&commit.id)
                .await?;

            let associated_prs = git_client.unwrap().associated_prs(&commit.id).await?;
            let pr_number = associated_prs.first().map(|pr| pr.number);
            commit.remote = RemoteContributor {
                username: remote_commit.username,
                pr_number: pr_number.and_then(|n| i64::try_from(n).ok()),
                ..RemoteContributor::default()
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
