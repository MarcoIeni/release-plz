mod args;
mod config;
mod log;
mod update_checker;
mod generate_schema;

use anyhow::Context;
use clap::Parser;
use release_plz_core::{ReleasePrRequest, ReleaseRequest};
use tracing::error;

use crate::args::{CliArgs, Command};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = CliArgs::parse();
    log::init(args.verbose);
    run(args).await.map_err(|e| {
        error!("{:?}", e);
        e
    })?;

    Ok(())
}

async fn run(args: CliArgs) -> anyhow::Result<()> {
    match args.command {
        Command::Update(cmd_args) => {
            let config = cmd_args.config()?;
            let update_request = cmd_args.update_request(config)?;
            let updates = release_plz_core::update(&update_request)?;
            println!("{}", updates.0.summary());
        }
        Command::ReleasePr(cmd_args) => {
            let config = cmd_args.update.config()?;
            let pr_labels = config.workspace.release_pr.pr_labels.clone();
            let pr_draft = config.workspace.release_pr.pr_draft;
            let update_request = cmd_args.update.update_request(config)?;
            let repo_url = update_request
                .repo_url()
                .context("can't determine repo url")?;
            let git = cmd_args
                .git_backend(repo_url.clone())
                .context("invalid git backend settings")?;
            let request = ReleasePrRequest::new(git, update_request)
                .mark_as_draft(pr_draft)
                .with_labels(pr_labels);
            release_plz_core::release_pr(&request).await?;
        }
        Command::Release(cmd_args) => {
            let config = cmd_args.config()?;
            let request: ReleaseRequest = cmd_args.release_request(config)?;
            release_plz_core::release(&request).await?;
        }
        Command::GenerateCompletions(cmd_args) => cmd_args.print(),
        Command::CheckUpdates => update_checker::check_update().await?,
        Command::GenerateSchema => {generate_schema::generate_schema()}
    }
    Ok(())
}
