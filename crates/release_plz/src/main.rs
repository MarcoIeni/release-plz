mod args;
mod log;

use std::path::{Path, PathBuf};

use anyhow::Context;
use cargo::core::SourceId;
use cargo_metadata::Package;
use clap::Parser;
use release_plz::{update_with_pr, Request, UpdateRequest};
use tempfile::tempdir;
use tracing::debug;

use crate::args::CliArgs;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    log::init();
    let args = CliArgs::parse();
    debug!("installing dependencies");
    debug!("dependencies installed");
    // TODO download in tmp directory
    //download_crate("rust-gh-example")?;
    let local_manifest_path = PathBuf::from("/home/marco/me/proj/rust-monorepo-example/Cargo.toml");
    // let local_manifest_path =
    //     fs::canonicalize(local_manifest_path).context("local_path doesn't exist")?;
    let remote_manifest_path = PathBuf::from("/home/marco/me/proj/rust-gh-example/Cargo.toml");
    let request = Request {
        github: args.github()?,
        update_request: UpdateRequest {
            local_manifest: &local_manifest_path,
            remote_manifest: &remote_manifest_path,
        },
    };
    update_with_pr(&request).await?;

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

fn download_crate(crates: &[&str]) -> anyhow::Result<Vec<Package>> {
    let config = cargo::Config::default().expect("Unable to get cargo config.");
    let source_id = SourceId::crates_io(&config).expect("Unable to retriece source id.");
    let crates: Vec<cargo_clone::Crate> = crates
        .iter()
        .map(|c| cargo_clone::Crate::new(c.to_string(), None))
        .collect();
    let temp_dir = tempdir()?;
    let directory = Some(temp_dir.as_ref().to_str().expect("invalid path"));
    let clone_opts = cargo_clone::CloneOpts::new(&crates, &source_id, directory, false);
    cargo_clone::clone(&clone_opts, &config).context("cannot download remote crates")?;
    Ok(list_crates(temp_dir.as_ref()))
}

fn list_crates(directory: &Path) -> Vec<Package> {
    cargo_edit::workspace_members(Some(directory)).unwrap()
}
