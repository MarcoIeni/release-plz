use std::path::PathBuf;

use cargo_metadata::Package;
use chrono::SecondsFormat;
use git_cmd::Repo;

use anyhow::{anyhow, Context};
use octocrab::OctocrabBuilder;
use secrecy::{ExposeSecret, SecretString};
use tracing::{info, instrument, Span};
use url::Url;

use crate::{copy_to_temp_dir, update, UpdateRequest, UpdateResult, CARGO_TOML};

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
    let local_manifest = new_manifest_dir.join(CARGO_TOML);
    let new_update_request = input
        .update_request
        .clone()
        .set_local_manifest(local_manifest)
        .context("can't find temporary project")?;
    let (packages_to_update, _repository) = update(&new_update_request)?;
    if !packages_to_update.is_empty() {
        let repo = Repo::new(new_manifest_dir)?;
        let pr = Pr::new(&packages_to_update);
        create_release_branch(&repo, &pr.branch)?;
        open_pr(&pr, &input.github).await?;
    }

    Ok(())
}

#[derive(Debug)]
struct Pr {
    branch: String,
    title: String,
}

impl Pr {
    fn new(packages_to_update: &[(Package, UpdateResult)]) -> Self {
        Self {
            branch: release_branch(),
            title: pr_title(packages_to_update),
        }
    }
}

fn release_branch() -> String {
    let now = chrono::offset::Utc::now();
    // Convert to a string of format "2018-01-26T18:30:09Z".
    let now = now.to_rfc3339_opts(SecondsFormat::Secs, true);
    // ':' is not a valid character for a branch name.
    let now = now.replace(':', "-");
    format!("release-plz/{now}")
}

fn pr_title(packages_to_update: &[(Package, UpdateResult)]) -> String {
    if packages_to_update.len() == 1 {
        let (package, update) = &packages_to_update[0];
        format!("chore({}): release v{}", package.name, update.version)
    } else {
        "chore: release".to_string()
    }
}

#[instrument(
    fields(
        default_branch = tracing::field::Empty,
    )
)]
async fn open_pr(pr: &Pr, github: &GitHub) -> anyhow::Result<()> {
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

    let pr = client
        .pulls(&github.owner, &github.repo)
        .create(&pr.title, &pr.branch, default_branch)
        .body("release-plz automatic bot")
        .send()
        .await?;

    if let Some(url) = pr.html_url {
        info!("opened pr: {}", url);
    }

    Ok(())
}

fn create_release_branch(repository: &Repo, release_branch: &str) -> anyhow::Result<()> {
    repository.checkout_new_branch(release_branch)?;
    repository.add_all_and_commit("chore: release")?;
    repository.push(release_branch)?;
    Ok(())
}
