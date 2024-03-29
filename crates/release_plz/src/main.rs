mod args;
mod changelog_config;
mod config;
mod generate_schema;
pub mod init;
mod log;
mod update_checker;

use anyhow::Context;
use args::OutputType;
use clap::Parser;
use release_plz_core::{ReleasePrRequest, ReleaseRequest};
use serde::Serialize;
use tracing::error;

use crate::args::{repo_command::RepoCommand as _, CliArgs, Command};

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
            let cargo_metadata = cmd_args.cargo_metadata()?;
            let config = cmd_args.config()?;
            let update_request = cmd_args.update_request(config, cargo_metadata)?;
            let updates = release_plz_core::update(&update_request)?;
            println!("{}", updates.0.summary());
        }
        Command::ReleasePr(cmd_args) => {
            let cargo_metadata = cmd_args.update.cargo_metadata()?;
            let config = cmd_args.update.config()?;
            let pr_labels = config.workspace.pr_labels.clone();
            let pr_draft = config.workspace.pr_draft;
            let update_request = cmd_args.update.update_request(config, cargo_metadata)?;
            let repo_url = update_request
                .repo_url()
                .context("can't determine repo url")?;
            let git = cmd_args
                .git_backend(repo_url.clone())
                .context("invalid git backend settings")?;
            let request = ReleasePrRequest::new(git, update_request)
                .mark_as_draft(pr_draft)
                .with_labels(pr_labels);
            let release_pr = release_plz_core::release_pr(&request).await?;
            if let Some(release_pr) = release_pr {
                cmd_args
                    .output
                    .map(|output_type| maybe_print_output(output_type, release_pr));
            }
        }
        Command::Release(cmd_args) => {
            let cargo_metadata = cmd_args.cargo_metadata()?;
            let config = cmd_args.config()?;
            let cmd_args_output = cmd_args.output;
            let request: ReleaseRequest = cmd_args.release_request(config, cargo_metadata)?;
            if let Some(release) = release_plz_core::release(&request).await? {
                cmd_args_output.map(|output_type| maybe_print_output(output_type, release));
            }
        }
        Command::GenerateCompletions(cmd_args) => cmd_args.print(),
        Command::CheckUpdates => update_checker::check_update().await?,
        Command::GenerateSchema => generate_schema::generate_schema_to_disk()?,
        Command::Init => init::init()?,
    }
    Ok(())
}

fn maybe_print_output(output_type: OutputType, output: impl Serialize) {
    match output_type {
        OutputType::Json => match serde_json::to_string(&output) {
            Ok(json) => println!("{json}"),
            Err(e) => tracing::error!("can't serialize release pr to json: {e}"),
        },
    }
}
