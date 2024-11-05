mod args;
mod changelog_config;
mod config;
mod generate_schema;
pub mod init;
mod log;
mod update_checker;

use args::{config_command::ConfigCommand as _, OutputType};
use clap::Parser;
use config::Config;
use release_plz_core::{ReleasePrRequest, ReleaseRequest, UpdateRequest};
use serde::Serialize;
use tracing::error;

use crate::args::{manifest_command::ManifestCommand as _, CliArgs, Command};

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
            let update_request = cmd_args.update_request(&config, cargo_metadata)?;
            let updates = release_plz_core::update(&update_request).await?;
            println!("{}", updates.0.summary());
        }
        Command::ReleasePr(cmd_args) => {
            anyhow::ensure!(
                cmd_args.update.git_token.is_some(),
                "please provide the git token with the --git-token cli argument."
            );
            let cargo_metadata = cmd_args.update.cargo_metadata()?;
            let config = cmd_args.update.config()?;
            let update_request = cmd_args.update.update_request(&config, cargo_metadata)?;
            let requests = get_release_pr_reqs(&config, update_request)?;

            let mut prs = vec![];
            for request in &requests {
                let release_pr = release_plz_core::release_pr(request).await?;

                if let Some(pr) = release_pr {
                    prs.push(pr);
                }
            }

            if let Some(output_type) = cmd_args.output {
                let prs_json = serde_json::json!({
                    "prs": prs
                });
                print_output(output_type, prs_json);
            }
        }
        Command::Release(cmd_args) => {
            let cargo_metadata = cmd_args.cargo_metadata()?;
            let config = cmd_args.config()?;
            let cmd_args_output = cmd_args.output;
            let request: ReleaseRequest = cmd_args.release_request(&config, cargo_metadata)?;
            let output = release_plz_core::release(&request)
                .await?
                .unwrap_or_default();
            if let Some(output_type) = cmd_args_output {
                print_output(output_type, output);
            }
        }
        Command::GenerateCompletions(cmd_args) => cmd_args.print(),
        Command::CheckUpdates => update_checker::check_update().await?,
        Command::GenerateSchema => generate_schema::generate_schema_to_disk()?,
        Command::Init => init::init()?,
        Command::SetVersion(cmd_args) => {
            let config = cmd_args.config()?;
            let request = cmd_args.set_version_request(&config)?;
            release_plz_core::set_version::set_version(&request)?;
        }
    }
    Ok(())
}

fn get_release_pr_reqs(
    config: &Config,
    update_request: UpdateRequest,
) -> anyhow::Result<Vec<ReleasePrRequest>> {
    if config.workspace.one_pr_per_package.unwrap_or(false)
        && update_request.single_package().is_none()
    {
        let packages = update_request.cargo_metadata().workspace_packages();

        packages
            .into_iter()
            .map(|package| {
                let update_request = update_request
                    .clone()
                    .with_single_package(package.name.clone());

                let release_pr_request = get_single_release_pr_req(config, update_request)?;

                let branch_prefix = release_pr_request.branch_prefix().to_owned();

                let release_pr_request = release_pr_request
                    .with_branch_prefix(Some(format!("{branch_prefix}{}-", package.name)));

                Ok(release_pr_request)
            })
            .collect()
    } else {
        let req = get_single_release_pr_req(config, update_request)?;
        Ok(vec![req])
    }
}

fn get_single_release_pr_req(
    config: &Config,
    update_request: UpdateRequest,
) -> anyhow::Result<ReleasePrRequest> {
    let pr_branch_prefix = config.workspace.pr_branch_prefix.clone();
    let pr_name = config.workspace.pr_name.clone();
    let pr_body = config.workspace.pr_body.clone();
    let pr_labels = config.workspace.pr_labels.clone();
    let pr_draft = config.workspace.pr_draft;
    let request = ReleasePrRequest::new(update_request)
        .mark_as_draft(pr_draft)
        .with_labels(pr_labels)
        .with_branch_prefix(pr_branch_prefix)
        .with_pr_name_template(pr_name)
        .with_pr_body_template(pr_body);
    Ok(request)
}

fn print_output(output_type: OutputType, output: impl Serialize) {
    match output_type {
        OutputType::Json => match serde_json::to_string(&output) {
            Ok(json) => println!("{json}"),
            Err(e) => tracing::error!("can't serialize release pr to json: {e}"),
        },
    }
}
