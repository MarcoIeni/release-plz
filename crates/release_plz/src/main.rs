mod args;
mod config;
mod log;
mod update_checker;

use anyhow::Context;
use clap::Parser;
use release_plz_core::{ReleasePrRequest, ReleaseRequest};
use tracing::error;

use crate::args::CliArgs;

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
    let config = args.config()?;
    match args.command {
        args::Command::Update(cmd_args) => {
            let update_request = cmd_args.update_request(config)?;
            let updates = release_plz_core::update(&update_request)?;
            println!("{}", updates.0.summary());
        }
        args::Command::ReleasePr(cmd_args) => {
            let pr_labels = config.workspace.release_pr.pr_labels.clone();
            let update_request = cmd_args.update.update_request(config)?;
            let repo_url = update_request
                .repo_url()
                .context("can't determine repo url")?;
            let git = cmd_args
                .git_backend(repo_url.clone())
                .context("invalid git backend settings")?;
            let request = ReleasePrRequest::new(git, update_request).with_labels(pr_labels);
            release_plz_core::release_pr(&request).await?;
        }
        args::Command::Release(cmd_args) => {
            let request: ReleaseRequest = cmd_args.release_request(config)?;
            release_plz_core::release(&request).await?;
        }
        args::Command::GenerateCompletions(cmd_args) => cmd_args.print(),
        args::Command::CheckUpdates => update_checker::check_update().await?,
    }
    Ok(())
}
