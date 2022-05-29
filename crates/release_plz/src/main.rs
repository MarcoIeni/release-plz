mod args;
mod log;

use anyhow::Context;
use clap::Parser;
use release_plz_core::{ReleasePrRequest, ReleaseRequest};

use crate::args::CliArgs;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = CliArgs::parse();
    log::init(args.verbose);

    match args.command {
        args::Command::Update(cmd_args) => {
            let update_request = cmd_args.update_request()?;
            release_plz_core::update(&update_request)?;
        }
        args::Command::ReleasePr(cmd_args) => {
            let update_request = cmd_args.update.update_request()?;
            let request = ReleasePrRequest {
                github: cmd_args.github().context("invalid github settings")?,
                update_request,
            };
            release_plz_core::release_pr(&request).await?;
        }
        args::Command::Release(cmd_args) => {
            let request: ReleaseRequest = cmd_args.into();
            release_plz_core::release(&request).await?;
        }
    }

    Ok(())
}
