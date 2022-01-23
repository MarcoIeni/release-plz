mod download;
mod git;
mod update;
mod version;
mod cmd;

pub use update::*;

use crate::git::Repo;

use anyhow::Context;
use fake::Fake;
use octocrab::OctocrabBuilder;
use secrecy::{ExposeSecret, SecretString};
use tracing::{instrument, Span};

/// Difference between local and remote crate
#[derive(Debug)]
struct Diff {
    pub commits: Vec<String>,
    /// Whether the crate name exists in the remote crates or not
    pub remote_crate_exists: bool,
}

impl Diff {
    fn new(remote_crate_exists: bool) -> Self {
        Self {
            commits: vec![],
            remote_crate_exists,
        }
    }

    fn should_update_version(&self) -> bool {
        self.remote_crate_exists && !self.commits.is_empty()
    }
}

#[derive(Debug)]
pub struct Request {
    pub github: GitHub,
    pub update_request: UpdateRequest,
}

#[derive(Debug)]
pub struct GitHub {
    pub owner: String,
    pub repo: String,
    pub token: SecretString,
}

/// Update a local rust project and raise a pull request
#[instrument]
pub async fn release_pr(input: &Request) -> anyhow::Result<()> {
    let (crates_to_update, repository) = next_versions(&input.update_request)?;
    if !crates_to_update.is_empty() {
        // TODO think about better naming
        let random_number: u64 = (100_000_000..999_999_999).fake();
        let release_branch = format!("release-{}", random_number);
        create_release_branch(&repository, &release_branch)?;
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
