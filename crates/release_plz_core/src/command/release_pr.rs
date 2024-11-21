use cargo_metadata::camino::Utf8Path;
use cargo_utils::CARGO_TOML;
use git_cmd::Repo;

use anyhow::Context;
use serde::Serialize;
use tracing::{debug, info, instrument};
use url::Url;

use crate::git::backend::{contributors_from_commits, BackendType, GitClient, GitPr, PrEdit};
use crate::git::github_graphql;
use crate::pr::{Pr, DEFAULT_BRANCH_PREFIX, OLD_BRANCH_PREFIX};
use crate::{
    copy_to_temp_dir, new_manifest_dir_path, new_project_root, publishable_packages_from_manifest,
    root_repo_path_from_manifest_dir, update, PackagesUpdate, UpdateRequest,
};

#[derive(Debug)]
pub struct ReleasePrRequest {
    /// Tera template for the release pull request name.
    pr_name_template: Option<String>,
    /// Tera template for the release pull request body.
    pr_body_template: Option<String>,
    /// If `true`, the created release PR will be marked as a draft.
    draft: bool,
    /// Labels to add to the release PR.
    labels: Vec<String>,
    /// PR Branch Prefix
    branch_prefix: String,
    pub update_request: UpdateRequest,
}

impl ReleasePrRequest {
    pub fn new(update_request: UpdateRequest) -> Self {
        Self {
            pr_name_template: None,
            pr_body_template: None,
            draft: false,
            labels: vec![],
            branch_prefix: DEFAULT_BRANCH_PREFIX.to_string(),
            update_request,
        }
    }

    pub fn with_pr_name_template(mut self, pr_name_template: Option<String>) -> Self {
        self.pr_name_template = pr_name_template;
        self
    }

    pub fn with_pr_body_template(mut self, pr_body_template: Option<String>) -> Self {
        self.pr_body_template = pr_body_template;
        self
    }

    pub fn with_labels(mut self, labels: Vec<String>) -> Self {
        self.labels = labels;
        self
    }

    pub fn mark_as_draft(mut self, draft: bool) -> Self {
        self.draft = draft;
        self
    }

    pub fn with_branch_prefix(mut self, pr_branch_prefix: Option<String>) -> Self {
        if let Some(branch_prefix) = pr_branch_prefix {
            self.branch_prefix = branch_prefix;
        }
        self
    }
}

/// Release pull request that release-plz opened/updated.
#[derive(Serialize, Debug)]
pub struct ReleasePr {
    /// The name of the branch where the changes are implemented.
    pub head_branch: String,
    /// The name of the branch the changes are pulled into.
    /// It is the default branch of the repository. E.g. `main`.
    pub base_branch: String,
    /// Url. Users can open it in the browser to see an html representation of the PR.
    pub html_url: Url,
    /// Number
    pub number: u64,
}

/// Open a pull request with the next packages versions of a local rust project
/// Returns:
/// - [`ReleasePr`] if release-plz opened or updated a PR.
/// - [`None`] if release-plz didn't open any pr. This happens when all packages
///   are up-to-date.
#[instrument(skip_all)]
pub async fn release_pr(input: &ReleasePrRequest) -> anyhow::Result<Option<ReleasePr>> {
    let manifest_dir = input.update_request.local_manifest_dir()?;
    let original_project_root = root_repo_path_from_manifest_dir(manifest_dir)?;
    let tmp_project_root_parent = copy_to_temp_dir(&original_project_root)?;
    let tmp_project_manifest_dir = new_manifest_dir_path(
        &original_project_root,
        manifest_dir,
        tmp_project_root_parent.path(),
    )?;

    let tmp_project_root =
        new_project_root(&original_project_root, tmp_project_root_parent.path())?;

    let local_manifest = tmp_project_manifest_dir.join(CARGO_TOML);
    let new_update_request = input
        .update_request
        .clone()
        .set_local_manifest(&local_manifest)
        .context("can't find temporary project")?;
    let (packages_to_update, _temp_repository) = update(&new_update_request)
        .await
        .context("failed to update packages")?;
    let git_client = input
        .update_request
        .git_client()?
        .context("can't find git client")?;
    if !packages_to_update.updates().is_empty() {
        let repo = Repo::new(tmp_project_root)?;
        let there_are_commits_to_push = repo.is_clean().is_err();
        if there_are_commits_to_push {
            let pr = open_or_update_release_pr(
                &local_manifest,
                &packages_to_update,
                &git_client,
                &repo,
                ReleasePrOptions {
                    draft: input.draft,
                    pr_name: input.pr_name_template.clone(),
                    pr_body: input.pr_body_template.clone(),
                    pr_labels: input.labels.clone(),
                    pr_branch_prefix: input.branch_prefix.clone(),
                },
            )
            .await?;
            return Ok(Some(pr));
        }
    }

    Ok(None)
}

struct ReleasePrOptions {
    draft: bool,
    pr_name: Option<String>,
    pr_body: Option<String>,
    pr_labels: Vec<String>,
    pr_branch_prefix: String,
}

async fn open_or_update_release_pr(
    local_manifest: &Utf8Path,
    packages_to_update: &PackagesUpdate,
    git_client: &GitClient,
    repo: &Repo,
    release_pr_options: ReleasePrOptions,
) -> anyhow::Result<ReleasePr> {
    let mut opened_release_prs = git_client
        .opened_prs(&release_pr_options.pr_branch_prefix)
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
            &release_pr_options.pr_branch_prefix,
            release_pr_options.pr_name,
            release_pr_options.pr_body,
        )
        .mark_as_draft(release_pr_options.draft)
        .with_labels(release_pr_options.pr_labels)
    };
    match opened_release_prs.first() {
        Some(opened_pr) => {
            handle_opened_pr(
                git_client,
                opened_pr,
                repo,
                &new_pr,
                &release_pr_options.pr_branch_prefix,
            )
            .await
        }
        None => create_pr(git_client, repo, &new_pr).await,
    }
}

async fn handle_opened_pr(
    git_client: &GitClient,
    opened_pr: &GitPr,
    repo: &Repo,
    new_pr: &Pr,
    branch_prefix: &str,
) -> Result<ReleasePr, anyhow::Error> {
    let pr_commits = git_client
        .pr_commits(opened_pr.number)
        .await
        .context("cannot get commits of release-plz pr")?;
    let pr_contributors = contributors_from_commits(&pr_commits);
    Ok(if pr_contributors.is_empty() {
        // There are no contributors, so we can force-push
        // in this PR, because we don't care about the git history.
        match update_pr(
            git_client,
            opened_pr,
            pr_commits.len(),
            repo,
            new_pr,
            branch_prefix,
        )
        .await
        {
            Ok(()) => ReleasePr {
                number: opened_pr.number,
                head_branch: opened_pr.branch().to_string(),
                html_url: opened_pr.html_url.clone(),
                base_branch: new_pr.base_branch.clone(),
            },
            Err(e) => {
                tracing::error!("cannot update release pr {}: {:?}. I'm closing the old release pr and opening a new one", opened_pr.number, e);
                git_client
                    .close_pr(opened_pr.number)
                    .await
                    .context("cannot close old release-plz prs")?;
                create_pr(git_client, repo, new_pr).await?
            }
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
        create_pr(git_client, repo, new_pr).await?
    })
}

async fn create_pr(git_client: &GitClient, repo: &Repo, pr: &Pr) -> anyhow::Result<ReleasePr> {
    repo.checkout_new_branch(&pr.branch)?;
    if matches!(git_client.backend, BackendType::Github) {
        github_create_release_branch(git_client, repo, &pr.branch, &pr.title).await?;
    } else {
        create_release_branch(repo, &pr.branch, &pr.title)?;
    }
    debug!("changes committed to release branch {}", pr.branch);

    let git_pr = git_client.open_pr(pr).await.context("Failed to open PR")?;
    Ok(ReleasePr {
        number: git_pr.number,
        head_branch: git_pr.branch().to_string(),
        html_url: git_pr.html_url,
        base_branch: pr.base_branch.clone(),
    })
}

async fn update_pr(
    git_client: &GitClient,
    opened_pr: &GitPr,
    commits_number: usize,
    repository: &Repo,
    new_pr: &Pr,
    branch_prefix: &str,
) -> anyhow::Result<()> {
    update_pr_branch(commits_number, opened_pr, repository, branch_prefix).with_context(|| {
        format!(
            "failed to update pr branch with changes from `{}` branch",
            repository.original_branch()
        )
    })?;
    if matches!(git_client.backend, BackendType::Github) {
        github_force_push(git_client, opened_pr, repository).await?;
    } else {
        force_push(opened_pr, repository)?;
    }
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
        git_client.edit_pr(opened_pr.number, pr_edit).await?;
    }
    info!("updated pr {}", opened_pr.html_url);
    Ok(())
}

/// Update the PR branch with the latest changes from the
/// original branch where release-plz was run (by default it's the default branch, e.g. `main`).
fn update_pr_branch(
    commits_number: usize,
    opened_pr: &GitPr,
    repository: &Repo,
    branch_prefix: &str,
) -> anyhow::Result<()> {
    // save local work
    repository.git(&["stash", "--include-untracked"])?;

    reset_branch(opened_pr, commits_number, repository, branch_prefix).inspect_err(|_e| {
        // restore local work
        if let Err(e) = repository.stash_pop() {
            tracing::error!("cannot restore local work: {:?}", e);
        }
    })?;
    repository.stash_pop()?;
    Ok(())
}

fn reset_branch(
    pr: &GitPr,
    commits_number: usize,
    repository: &Repo,
    branch_prefix: &str,
) -> anyhow::Result<()> {
    // sanity check to avoid doing bad things on non-release-plz branches
    anyhow::ensure!(
        pr.branch().starts_with(branch_prefix)
            || pr.branch().starts_with(DEFAULT_BRANCH_PREFIX)
            || pr.branch().starts_with(OLD_BRANCH_PREFIX),
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
    add_changes_and_commit(repository, &pr.title)?;
    repository.force_push(pr.branch())?;
    Ok(())
}

async fn github_force_push(
    client: &GitClient,
    pr: &GitPr,
    repository: &Repo,
) -> anyhow::Result<()> {
    // Create a temporary branch.
    let tmp_release_branch = {
        let name = format!("{}-tmp-{}", pr.branch(), rand::random::<u32>());
        TmpBranch::checkout_new(repository, name)
    }?;

    // Push the "Verified" commit in the temporary branch using
    // the GitHub API.
    // We push the release-plz changes to the temporary branch instead of the release PR branch because:
    // - You can't force-push with the GitHub API, so we can't commit to the release PR branch
    //   directly if we want a "Verified" commit.
    // - If we revert the last commit of the release PR branch, GitHub will close the release PR
    //   because the branch is the same as the default branch. So we can't revert the latest release-plz commit and push the new one.
    // To learn more, see https://github.com/release-plz/release-plz/issues/1487
    github_create_release_branch(client, repository, &tmp_release_branch.name, &pr.title).await?;

    repository.fetch(&tmp_release_branch.name)?;

    // Rewrite the PR branch so that it's the same as the temporary branch.
    repository.force_push(&format!(
        "{}/{}:{}",
        repository.original_remote(),
        tmp_release_branch.name,
        pr.branch()
    ))?;

    // The temporary branch is deleted in remote when it goes out of scope.
    Ok(())
}

/// Temporary branch.
/// It deletes the branch in remote when it goes out of scope.
/// In this way, we can ensure that the branch is deleted even if the program panics.
struct TmpBranch<'a> {
    name: String,
    repository: &'a Repo,
}

impl<'a> TmpBranch<'a> {
    fn checkout_new(repository: &'a Repo, name: impl Into<String>) -> anyhow::Result<Self> {
        let name = name.into();
        repository.checkout_new_branch(&name)?;
        let branch = Self { name, repository };
        Ok(branch)
    }
}

impl Drop for TmpBranch<'_> {
    fn drop(&mut self) {
        if let Err(e) = self.repository.delete_branch_in_remote(&self.name) {
            tracing::error!("cannot delete branch {}: {:?}", self.name, e);
        }
    }
}

fn create_release_branch(
    repository: &Repo,
    release_branch: &str,
    commit_message: &str,
) -> anyhow::Result<()> {
    add_changes_and_commit(repository, commit_message)?;
    repository.push(release_branch)?;
    Ok(())
}

async fn github_create_release_branch(
    client: &GitClient,
    repository: &Repo,
    release_branch: &str,
    commit_message: &str,
) -> anyhow::Result<()> {
    repository.push(release_branch)?;
    github_graphql::commit_changes(client, repository, commit_message, release_branch).await
}

fn add_changes_and_commit(repository: &Repo, commit_message: &str) -> anyhow::Result<()> {
    let changes_expect_typechanges = repository.changes_except_typechanges()?;
    repository.add(&changes_expect_typechanges)?;
    repository.commit_signed(commit_message)?;
    Ok(())
}
