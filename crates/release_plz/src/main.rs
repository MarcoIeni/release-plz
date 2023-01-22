mod args;
mod log;

use anyhow::Context;
use clap::Parser;
use release_plz_core::{update_checker::get_latest_version, ReleasePrRequest, ReleaseRequest};
use tracing::error;

use crate::args::CliArgs;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    const VERSION: &str = env!("CARGO_PKG_VERSION");
    if !get_latest_version().await.is_empty() && &get_latest_version().await[13..] != VERSION {
        println!("A newer version ({}) is available at https://github.com/MarcoIeni/release-plz\nPlease Update.", get_latest_version().await);
    }
    let args = CliArgs::parse();
    log::init(args.verbose);
    run(args).await.map_err(|e| {
        error!("{}", e);
        e
    })?;

    Ok(())
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
