use std::path::Path;

use git_cmd::Repo;

use anyhow::Context;
use tracing::{info, instrument};

use crate::git::backend::{contributors_from_commits, BackendType, GitClient, GitPr, PrEdit};
use crate::git::github_graphql;
use crate::pr::{Pr, BRANCH_PREFIX, OLD_BRANCH_PREFIX};
use crate::{
    copy_to_temp_dir, new_manifest_dir_path, new_project_root, publishable_packages_from_manifest,
    root_repo_path_from_manifest_dir, update, GitBackend, PackagesUpdate, UpdateRequest,
    CARGO_TOML,
};

#[derive(Debug)]
pub struct ReleasePrRequest {
    pub git: GitBackend,
    /// If `true`, the created release PR will be marked as a draft.
    draft: bool,
    /// Labels to add to the release PR.
    labels: Vec<String>,
    pub update_request: UpdateRequest,
}

impl ReleasePrRequest {
    pub fn new(git: GitBackend, update_request: UpdateRequest) -> Self {
        Self {
            git,
            draft: false,
            labels: vec![],
            update_request,
        }
    }

    pub fn with_labels(mut self, labels: Vec<String>) -> Self {
        self.labels = labels;
        self
    }

    pub fn mark_as_draft(mut self, draft: bool) -> Self {
        self.draft = draft;
        self
    }
}

/// Open a pull request with the next packages versions of a local rust project
#[instrument(skip_all)]
pub async fn release_pr(input: &ReleasePrRequest) -> anyhow::Result<()> {
    let manifest_dir = input.update_request.local_manifest_dir()?;
    let original_project_root = root_repo_path_from_manifest_dir(manifest_dir)?;
    let tmp_project_root_parent = copy_to_temp_dir(&original_project_root)?;
    let tmp_project_manifest_dir = new_manifest_dir_path(
        &original_project_root,
        manifest_dir,
        tmp_project_root_parent.as_ref(),
    )?;

    let tmp_project_root =
        new_project_root(&original_project_root, tmp_project_root_parent.as_ref())?;

    let local_manifest = tmp_project_manifest_dir.join(CARGO_TOML);
    let new_update_request = input
        .update_request
        .clone()
        .set_local_manifest(&local_manifest)
        .context("can't find temporary project")?;
    let (packages_to_update, _temp_repository) =
        update(&new_update_request).context("failed to update packages")?;
    let git_client = GitClient::new(input.git.clone())?;
    if !packages_to_update.updates().is_empty() {
        let repo = Repo::new(tmp_project_root)?;
        let there_are_commits_to_push = repo.is_clean().is_err();
        if there_are_commits_to_push {
            open_or_update_release_pr(
                &local_manifest,
                &packages_to_update,
                &git_client,
                &repo,
                input.draft,
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
    draft: bool,
    pr_labels: Vec<String>,
) -> anyhow::Result<()> {
    let mut opened_release_prs = git_client
        .opened_prs(BRANCH_PREFIX)
        .await
        .context("cannot get opened release-plz prs")?;

    // Check if there are opened release-plz prs with the old prefix.
    // This ensures retro-compatibility with the release-plz versions.
    // TODO: Remove this check on release-plz v0.4.0.
    if opened_release_prs.is_empty() {
        opened_release_prs = git_client
            .opened_prs(OLD_BRANCH_PREFIX)
            .await
            .context("cannot get opened release-plz prs")?;
    }

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
            publishable_packages_from_manifest(local_manifest)?.len() > 1;
        Pr::new(
            repo.original_branch(),
            packages_to_update,
            project_contains_multiple_pub_packages,
        )
        .mark_as_draft(draft)
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
    if matches!(git_client.backend, BackendType::Github) {
        github_create_release_branch(git_client, repo, &pr.branch).await?;
    } else {
        create_release_branch(repo, &pr.branch)?;
    }
    git_client.open_pr(pr).await.context("Failed to open PR")?;
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
    anyhow::ensure!(
        pr.branch().starts_with(BRANCH_PREFIX) || pr.branch().starts_with(OLD_BRANCH_PREFIX),
        "wrong branch name"
    );

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
    add_changes_and_commit(repository)?;
    repository.force_push(pr.branch())?;
    Ok(())
}

fn create_release_branch(repository: &Repo, release_branch: &str) -> anyhow::Result<()> {
    repository.checkout_new_branch(release_branch)?;
    add_changes_and_commit(repository)?;
    repository.push(release_branch)?;
    Ok(())
}

async fn github_create_release_branch(
    client: &GitClient,
    repository: &Repo,
    release_branch: &str,
) -> anyhow::Result<()> {
    repository.checkout_new_branch(release_branch)?;
    repository.push(release_branch)?;
    github_graphql::commit_changes(client, repository, "chore: release", release_branch).await
}

fn add_changes_and_commit(repository: &Repo) -> anyhow::Result<()> {
    let changes_expect_typechanges = repository.changes_except_typechanges()?;
    repository.add(&changes_expect_typechanges)?;
    repository.commit_signed("chore: release")?;
    Ok(())
}
