use std::path::{Path, PathBuf};

use cargo_metadata::Package;
use git_cmd::Repo;

use anyhow::{anyhow, Context};
use tracing::instrument;

use crate::backend::GitClient;
use crate::github_client::{contributors_from_commits, GitHubPr, PrCommit};
use crate::pr::{Pr, BRANCH_PREFIX};
use crate::{
    copy_to_temp_dir, publishable_packages, update, GitBackend, UpdateRequest, UpdateResult,
    CARGO_TOML,
};

#[derive(Debug)]
pub struct ReleasePrRequest {
    pub git: GitBackend,
    pub update_request: UpdateRequest,
}

/// Open a pull request with the next packages versions of a local rust project
#[instrument]
pub async fn release_pr(input: &ReleasePrRequest) -> anyhow::Result<()> {
    let manifest_dir = input.update_request.local_manifest_dir()?;
    let tmp_project_root = copy_to_temp_dir(manifest_dir)?;
    let manifest_dir_name = manifest_dir
        .iter()
        .last()
        .ok_or_else(|| anyhow!("wrong local manifest path"))?;
    let manifest_dir_name = PathBuf::from(manifest_dir_name);
    let new_manifest_dir = tmp_project_root.as_ref().join(manifest_dir_name);
    let local_manifest = new_manifest_dir.join(CARGO_TOML);
    let new_update_request = input
        .update_request
        .clone()
        .set_local_manifest(&local_manifest)
        .context("can't find temporary project")?;
    let (packages_to_update, _temp_repository) = update(&new_update_request)?;
    let git_client = GitClient::new(&input.git)?;
    if !packages_to_update.is_empty() {
        let repo = Repo::new(new_manifest_dir)?;
        let there_are_commits_to_push = repo.is_clean().is_err();
        if there_are_commits_to_push {
            match &git_client {
                GitClient::GitHub(gh_client) => {
                    let opened_release_prs = gh_client
                        .opened_prs(BRANCH_PREFIX)
                        .await
                        .context("cannot get opened release-plz prs")?;
                    // Close old release-plz prs, except one.
                    for pr in &opened_release_prs[1..] {
                        gh_client
                            .close_pr(pr.number)
                            .await
                            .context("cannot close old release-plz prs")?;
                    }

                    match opened_release_prs.first() {
                        Some(pr) => {
                            let pr_commits = gh_client
                                .pr_commits(pr.number)
                                .await
                                .context("cannot get commits of release-plz pr")?;
                            let pr_contributors = contributors_from_commits(&pr_commits);
                            if pr_contributors.is_empty() {
                                let update_outcome = update_pr(pr, &pr_commits[0], &repo);
                                if let Err(e) = update_outcome {
                                    tracing::error!("cannot update release-plz pr: {}", e);
                                    gh_client
                                        .close_pr(pr.number)
                                        .await
                                        .context("cannot close old release-plz prs")?;
                                    create_pr(
                                        &git_client,
                                        &repo,
                                        &packages_to_update,
                                        &local_manifest,
                                    )
                                    .await?
                                }
                            } else {
                                // There's a contributor, so we don't want to force-push in this PR.
                                // We close it because we want to save the contributor's work.
                                // TODO improvement: check how many lines the commit added, if no lines (for example a merge to update the branch),
                                //      then don't count it as a contributor.
                                gh_client
                                    .close_pr(pr.number)
                                    .await
                                    .context("cannot close old release-plz prs")?;
                            }
                        }
                        None => {
                            create_pr(&git_client, &repo, &packages_to_update, &local_manifest)
                                .await?
                        }
                    }
                }
                GitClient::Gitea(_) => {
                    close_old_prs(&git_client).await?;
                    create_pr(&git_client, &repo, &packages_to_update, &local_manifest).await?;
                }
            }
        }
    }

    Ok(())
}

async fn close_old_prs(git_client: &GitClient<'_>) -> anyhow::Result<()> {
    git_client
        .close_prs_on_branches(BRANCH_PREFIX)
        .await
        .context("cannot close old release-plz prs")?;
    Ok(())
}

async fn create_pr(
    git_client: &GitClient<'_>,
    repo: &Repo,
    packages_to_update: &[(Package, UpdateResult)],
    local_manifest: &Path,
) -> anyhow::Result<()> {
    let project_contains_multiple_pub_packages = publishable_packages(local_manifest)?.len() > 1;
    let pr = Pr::new(
        repo.default_branch(),
        packages_to_update,
        project_contains_multiple_pub_packages,
    );
    create_release_branch(repo, &pr.branch)?;
    git_client.open_pr(&pr).await?;
    Ok(())
}

fn update_pr(pr: &GitHubPr, first_pr_commit: &PrCommit, repository: &Repo) -> anyhow::Result<()> {
    // save local work
    repository.git(&["stash"])?;
    // sanity check to avoid doing bad things on the default branch
    anyhow::ensure!(pr.branch().starts_with(BRANCH_PREFIX), "wrong branch name");
    let parent_sha = first_pr_commit
        .parent()
        .context("can't determine parent sha")?;
    repository.checkout(parent_sha)?;
    repository.git(&["stash", "pop"])?;
    let changes_expect_typechanges = repository.changes_except_typechanges()?;
    repository.add(&changes_expect_typechanges)?;
    repository.commit("chore: release")?;
    repository.force_push(pr.branch())?;
    Ok(())
}

fn create_release_branch(repository: &Repo, release_branch: &str) -> anyhow::Result<()> {
    repository.checkout_new_branch(release_branch)?;
    let changes_expect_typechanges = repository.changes_except_typechanges()?;
    repository.add(&changes_expect_typechanges)?;
    repository.commit("chore: release")?;
    repository.push(release_branch)?;
    Ok(())
}
