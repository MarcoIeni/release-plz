mod args;
mod log;

use std::{path::PathBuf, process::Command};

use clap::Parser;
use release_plz::{update, GitHub, Request};
use tracing::debug;

use crate::args::CliArgs;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    log::init();
    let args = CliArgs::parse();
    debug!("installing dependencies");
    //install_dependencies()?;
    debug!("dependencies installed");
    // TODO download in tmp directory
    //download_crate("rust-gh-example")?;
    let local_manifest_path = PathBuf::from("/home/marco/me/proj/rust-gh-example2/Cargo.toml");
    // let local_manifest_path =
    //     fs::canonicalize(local_manifest_path).context("local_path doesn't exist")?;
    let remote_manifest_path = PathBuf::from("/home/marco/me/proj/rust-gh-example/Cargo.toml");
    let request = Request {
        github: GitHub {
            repo_url: args.repo_url,
            token: args.github_token,
        },
        local_manifest: &local_manifest_path,
        remote_manifest: &remote_manifest_path,
    };
    update(&request).await?;

    // pr command:
    // - go back commit by commit and for every local crate:
    //   - If the local crate was edited in that commit:
    //     - if the hash of that crate is the same of the remote crate, that local crate is done.
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

fn install_dependencies() -> anyhow::Result<()> {
    for program in ["cargo-clone"] {
        Command::new("cargo").args(["install", program]).output()?;
    }
    Ok(())
}

fn download_crate(crate_name: &str) -> anyhow::Result<()> {
    Command::new("cargo").args(["clone", crate_name]).output()?;
    Ok(())
}
