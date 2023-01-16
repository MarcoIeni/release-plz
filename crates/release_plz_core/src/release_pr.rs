use std::path::PathBuf;

use git_cmd::Repo;

use anyhow::{anyhow, Context};
use tracing::instrument;

use crate::backend::GitClient;
use crate::pr::{Pr, BRANCH_PREFIX};
use crate::{
    copy_to_temp_dir, publishable_packages, update, GitBackend, UpdateRequest, CARGO_TOML,
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
    let gh_client = GitClient::new(&input.git)?;
    gh_client
        .close_prs_on_branches(BRANCH_PREFIX)
        .await
        .context("cannot close old release-plz prs")?;
    if !packages_to_update.is_empty() {
        let repo = Repo::new(new_manifest_dir)?;
        let there_are_commits_to_push = repo.is_clean().is_err();
        if there_are_commits_to_push {
            let project_contains_multiple_pub_packages =
                publishable_packages(local_manifest)?.len() > 1;
            let pr = Pr::new(
                repo.default_branch(),
                packages_to_update.as_ref(),
                project_contains_multiple_pub_packages,
            );
            create_release_branch(&repo, &pr.branch)?;
            gh_client.open_pr(&pr).await?;
        }
    }

    Ok(())
}

fn create_release_branch(repository: &Repo, release_branch: &str) -> anyhow::Result<()> {
    repository.checkout_new_branch(release_branch)?;
    let changes_expect_typechanges = repository.changes_expect_typechanges()?;
    repository.add(&changes_expect_typechanges)?;
    repository.commit("chore: release")?;
    repository.push(release_branch)?;
    Ok(())
}
