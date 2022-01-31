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

    // pr command:
    // - go back commit by commit and for every local crate:
    //   - If the local crate was edited in that commit:
    //     - if that crate is the same of the remote crate, that local crate is done.
    //     - otherwise:
    //       - add the entry to the changelog of that crate.
    //       - bump the version of that crate according to the semantic versioning of the commit.
    // - raise PR

    Ok(())
}
