use std::path::{Path, PathBuf};

use git_cmd::Repo;

use anyhow::{anyhow, Context};
use tracing::{info, instrument};

use crate::git::backend::{contributors_from_commits, GitClient, GitPr, PrEdit};
use crate::pr::{Pr, BRANCH_PREFIX};
use crate::{
    copy_to_temp_dir, publishable_packages, update, GitBackend, PackagesUpdate, UpdateRequest,
    CARGO_TOML,
};

#[derive(Debug)]
pub struct ReleasePrRequest {
    pub git: GitBackend,
    /// Labels to add to the release PR.
    labels: Vec<String>,
    pub update_request: UpdateRequest,
}

impl ReleasePrRequest {
    pub fn new(git: GitBackend, update_request: UpdateRequest) -> Self {
        Self {
            git,
            labels: vec![],
            update_request,
        }
    }

    pub fn with_labels(mut self, labels: Vec<String>) -> Self {
        self.labels = labels;
        self
    }
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
    let git_client = GitClient::new(input.git.clone())?;
    if !packages_to_update.updates.is_empty() {
        let repo = Repo::new(new_manifest_dir)?;
        let there_are_commits_to_push = repo.is_clean().is_err();
        if there_are_commits_to_push {
            open_or_update_release_pr(
                &local_manifest,
                &packages_to_update,
                &git_client,
                &repo,
                input.labels.clone(),
            )
            .await?;
        }
    }

    Ok(())
}

async fn open_or_update_release_pr(
    local_manifest: &Path,
    packages_to_update: &PackagesUpdate,
    git_client: &GitClient,
    repo: &Repo,
    pr_labels: Vec<String>,
) -> anyhow::Result<()> {
    let opened_release_prs = git_client
        .opened_prs(BRANCH_PREFIX)
        .await
        .context("cannot get opened release-plz prs")?;
    // Close all release-plz prs, except one.
    let old_release_prs = opened_release_prs.iter().skip(1);
    for pr in old_release_prs {
        git_client
            .close_pr(pr.number)
            .await
            .context("cannot close old release-plz prs")?;
    }

    let new_pr = {
        let project_contains_multiple_pub_packages =
            publishable_packages(local_manifest)?.len() > 1;
        Pr::new(
            repo.original_branch(),
            packages_to_update,
            project_contains_multiple_pub_packages,
        )
        .with_labels(pr_labels)
    };
    match opened_release_prs.first() {
        Some(opened_pr) => {
            let pr_commits = git_client
                .pr_commits(opened_pr.number)
                .await
                .context("cannot get commits of release-plz pr")?;
            let pr_contributors = contributors_from_commits(&pr_commits);
            if pr_contributors.is_empty() {
                // There are no contributors, so we can force-push
                // in this PR, because we don't care about the git history.
                let update_outcome =
                    update_pr(git_client, opened_pr, pr_commits.len(), repo, &new_pr).await;
                if let Err(e) = update_outcome {
                    tracing::error!("cannot update release pr {}: {:?}. I'm closing the old release pr and opening a new one", opened_pr.number, e);
                    git_client
                        .close_pr(opened_pr.number)
                        .await
                        .context("cannot close old release-plz prs")?;
                    create_pr(git_client, repo, &new_pr).await?
                }
            } else {
                // There's a contributor, so we don't want to force-push in this PR.
                // We close it because we want to save the contributor's work.
                // TODO improvement: check how many lines the commit added, if no lines (for example a merge to update the branch),
                //      then don't count it as a contributor.
                info!("closing pr {} to preserve git history", opened_pr.html_url);
                git_client
                    .close_pr(opened_pr.number)
                    .await
                    .context("cannot close old release-plz prs")?;
                create_pr(git_client, repo, &new_pr).await?
            }
        }
        None => create_pr(git_client, repo, &new_pr).await?,
    }
    Ok(())
}

async fn create_pr(git_client: &GitClient, repo: &Repo, pr: &Pr) -> anyhow::Result<()> {
    create_release_branch(repo, &pr.branch)?;
    git_client.open_pr(pr).await?;
    Ok(())
}

async fn update_pr(
    git_client: &GitClient,
    opened_pr: &GitPr,
    commits_number: usize,
    repository: &Repo,
    new_pr: &Pr,
) -> anyhow::Result<()> {
    // save local work
    repository.git(&["stash", "--include-untracked"])?;

    reset_branch(opened_pr, commits_number, repository).map_err(|e| {
        // restore local work
        if let Err(e) = repository.stash_pop() {
            tracing::error!("cannot restore local work: {:?}", e);
        }
        e
    })?;
    repository.stash_pop()?;
    force_push(opened_pr, repository)?;
    let pr_edit = {
        let mut pr_edit = PrEdit::new();
        if opened_pr.title != new_pr.title {
            pr_edit = pr_edit.with_title(new_pr.title.clone());
        }
        if opened_pr.body.as_ref() != Some(&new_pr.body) {
            pr_edit = pr_edit.with_body(new_pr.body.clone());
        }
        pr_edit
    };
    if pr_edit.contains_edit() {
        git_client.edit_pr(opened_pr.number, &pr_edit).await?;
    }
    info!("updated pr {}", opened_pr.html_url);
    Ok(())
}

fn reset_branch(pr: &GitPr, commits_number: usize, repository: &Repo) -> anyhow::Result<()> {
    // sanity check to avoid doing bad things on non-release-plz branches
    anyhow::ensure!(pr.branch().starts_with(BRANCH_PREFIX), "wrong branch name");

    if repository.checkout(pr.branch()).is_err() {
        repository.git(&["pull"])?;
        repository.checkout(pr.branch())?;
    };

    let head = format!("HEAD~{commits_number}");
    repository.git(&["reset", "--hard", &head])?;

    repository.fetch(repository.original_branch())?;

    // Update PR branch with latest changes from the default branch.
    if let Err(e) = repository.git(&["rebase", repository.original_branch()]) {
        // Get back to the state before "git rebase" to clean the merge conflict.
        repository.git(&["rebase ", "--abort"])?;
        return Err(e.context("cannot rebase from default branch"));
    }

    Ok(())
}

fn force_push(pr: &GitPr, repository: &Repo) -> anyhow::Result<()> {
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
