mod args;
mod log;

use anyhow::Context;
use clap::Parser;
use release_plz_core::{ReleasePrRequest, ReleaseRequest};
use tracing::error;

use crate::args::CliArgs;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    if !check_updates().await.0 {
        println!("A newer version ({}) is available at https://github.com/MarcoIeni/release-plz\nPlease Update.", check_updates().await.1);
    }
    let args = CliArgs::parse();
    log::init(args.verbose);
    run(args).await.map_err(|e| {
        error!("{}", e);
        e
    })?;

    Ok(())
}

async fn check_updates() -> (bool, String) {
    const VERSION: &str = env!("CARGO_PKG_VERSION");
    if let Ok(latest_release) = octocrab::instance()
        .repos("MarcoIeni", "release-plz")
        .releases()
        .get_latest()
        .await
    {
        let tag_name = latest_release.tag_name;
        (VERSION == &tag_name[1..], tag_name)
    } else {
        (true, "".to_string())
    }
}

async fn run(args: CliArgs) -> anyhow::Result<()> {
    match args.command {
        args::Command::Update(cmd_args) => {
            let update_request = cmd_args.update_request()?;
            release_plz_core::update(&update_request)?;
        }
        args::Command::ReleasePr(cmd_args) => {
            let update_request = cmd_args.update.update_request()?;
            let request = ReleasePrRequest {
                git: cmd_args
                    .git_backend()
                    .context("invalid git backend settings")?,
                update_request,
            };
            release_plz_core::release_pr(&request).await?;
        }
        args::Command::Release(cmd_args) => {
            let request: ReleaseRequest = cmd_args.try_into()?;
            release_plz_core::release(&request).await?;
        }
        args::Command::GenerateCompletions(cmd_args) => cmd_args.print(),
    }
    Ok(())
}
