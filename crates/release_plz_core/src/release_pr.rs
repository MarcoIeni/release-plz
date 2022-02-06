use git_cmd::Repo;

use anyhow::Context;
use fake::Fake;
use octocrab::OctocrabBuilder;
use secrecy::{ExposeSecret, SecretString};
use tracing::{instrument, Span};

use crate::{next_versions, UpdateRequest};

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
}

/// Open a pull request with the next packages versions of a local rust project
#[instrument]
pub async fn release_pr(input: &ReleasePrRequest) -> anyhow::Result<()> {
    let (packages_to_update, repository) = next_versions(&input.update_request)?;
    if !packages_to_update.is_empty() {
        let random_number: u64 = (100_000_000..999_999_999).fake();
        let release_branch = format!("release-{}", random_number);
        create_release_branch(&repository.repo, &release_branch)?;
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
    let client = OctocrabBuilder::new()
        .personal_token(github.token.expose_secret().clone())
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
