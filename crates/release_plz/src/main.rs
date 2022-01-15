mod args;
mod log;

use anyhow::Context;
use clap::Parser;
use release_plz::{update_with_pr, Request};

use crate::args::CliArgs;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    log::init();
    let args = CliArgs::parse();

    match args.command {
        args::Command::Update(cmd_args) => {
            let update_request = cmd_args.update_request();
            release_plz::update(&update_request)?;
        }
        args::Command::UpdateWithPr(cmd_args) => {
            let update_request = cmd_args.update.update_request();
            let request = Request {
                github: cmd_args.github().context("invalid github settings")?,
                update_request,
            };
            update_with_pr(&request).await?;
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

    // release command (probably this is already done in ):
    // - for every local_crate with a version != remote one:
    //   - publish crate
    //   - create a new tag with format `local_crate v*new_version*`
    // // Maybe the same or similar is done by :
    // // cargo workspaces publish  --from-git --token "${TOKEN}" --yes
    Ok(())
}
