#![doc = include_str!("../../../README.md")]

mod args;
mod log;

use anyhow::Context;
use clap::Parser;
use release_plz_core::ReleasePrRequest;

use crate::args::CliArgs;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    log::init();
    let args = CliArgs::parse();

    match args.command {
        args::Command::Update(cmd_args) => {
            let update_request = cmd_args.update_request();
            release_plz_core::update(&update_request)?;
        }
        args::Command::ReleasePr(cmd_args) => {
            let update_request = cmd_args.update.update_request();
            let request = ReleasePrRequest {
                github: cmd_args.github().context("invalid github settings")?,
                update_request,
            };
            release_plz_core::release_pr(&request).await?;
        }
    }

    Ok(())
}
