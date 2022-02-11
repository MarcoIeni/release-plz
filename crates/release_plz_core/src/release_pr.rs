use std::path::PathBuf;

use git_cmd::Repo;

use anyhow::{anyhow, Context};
use fake::Fake;
use octocrab::OctocrabBuilder;
use secrecy::{ExposeSecret, SecretString};
use tracing::{instrument, Span};
use url::Url;

use crate::{copy_to_temp_dir, update, UpdateRequest, CARGO_TOML};

#[derive(Debug)]
pub struct ReleasePrRequest {
    pub github: GitHub,
    pub update_request: UpdateRequest,
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

/// Open a pull request with the next packages versions of a local rust project
#[instrument]
pub async fn release_pr(input: &ReleasePrRequest) -> anyhow::Result<()> {
    let manifest_dir = input
        .update_request
        .local_manifest()
        .parent()
        .ok_or_else(|| anyhow!("wrong local manifest path"))?;
    let tmp_project_root = copy_to_temp_dir(manifest_dir)?;
    let manifest_dir_name = manifest_dir
        .iter()
        .last()
        .ok_or_else(|| anyhow!("wrong local manifest path"))?;
    let manifest_dir_name = PathBuf::from(manifest_dir_name);
    let new_manifest_dir = tmp_project_root.as_ref().join(manifest_dir_name);
    let new_update_request = {
        let mut ur = UpdateRequest::new(new_manifest_dir.join(CARGO_TOML))
            .context("can't find temporary project")?;
        if let Some(remote) = input.update_request.remote_manifest() {
            ur = ur.with_remote_manifest(remote.to_path_buf())?;
        }
        ur
    };
    let (packages_to_update, _repository) = update(&new_update_request)?;
    if !packages_to_update.is_empty() {
        let random_number: u64 = (100_000_000..999_999_999).fake();
        let release_branch = format!("release-{}", random_number);
        let repo = Repo::new(new_manifest_dir)?;
        create_release_branch(&repo, &release_branch)?;
        open_pr(&release_branch, &input.github).await?;
    }

    Ok(())
}

#[instrument(
    fields(
        default_branch = tracing::field::Empty,
    )
)]
async fn open_pr(release_branch: &str, github: &GitHub) -> anyhow::Result<()> {
    let mut octocrab_builder =
        OctocrabBuilder::new().personal_token(github.token.expose_secret().clone());

    if let Some(base_url) = &github.base_url {
        octocrab_builder = octocrab_builder
            .base_url(base_url.clone())
            .context("Invalid GitHub base url")?;
    }

    let client = octocrab_builder
        .build()
        .context("Failed to build GitHub client")?;

    let default_branch = client
        .repos(&github.owner, &github.repo)
        .get()
        .await
        .context(format!(
            "failed to retrieve GitHub repository {}/{}",
            github.owner, github.repo
        ))?
        .default_branch
        .context("failed to retrieve default branch")?;
    Span::current().record("default_branch", &default_branch.as_str());

    let _pr = client
        .pulls(&github.owner, &github.repo)
        .create("chore: release", release_branch, default_branch)
        .body("release-plz automatic bot")
        .send()
        .await?;

    Ok(())
}

fn create_release_branch(repository: &Repo, release_branch: &str) -> anyhow::Result<()> {
    repository.checkout_new_branch(release_branch)?;
    repository.add_all_and_commit("chore: release")?;
    repository.push(release_branch)?;
    Ok(())
}
