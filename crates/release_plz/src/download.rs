use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{anyhow, Context};
use cargo::core::SourceId;
use cargo_metadata::Package;
use tempfile::tempdir;
use tracing::instrument;

#[instrument]
pub fn download_crates(crates: &[&str]) -> anyhow::Result<Vec<Package>> {
    let config = cargo::Config::default().expect("Unable to get cargo config.");
    let source_id = SourceId::crates_io(&config).expect("Unable to retriece source id.");
    let crates: Vec<cargo_clone::Crate> = crates
        .iter()
        .map(|c| cargo_clone::Crate::new(c.to_string(), None))
        .collect();
    let temp_dir = tempdir()?;
    let directory = temp_dir.as_ref().to_str().expect("invalid path");
    let clone_opts = cargo_clone::CloneOpts::new(&crates, &source_id, Some(directory), false);
    cargo_clone::clone(&clone_opts, &config).context("cannot download remote crates")?;
    let crates = if crates.len() == 1 {
        vec![read_package(directory)?]
    } else {
        let crates = sub_directories(directory)?;
        let crates = crates
            .iter()
            .map(|p| read_package(&p))
            .collect::<Result<Vec<Package>, _>>();
        crates?
    };
    Ok(crates)
}

fn sub_directories(directory: impl AsRef<Path>) -> anyhow::Result<Vec<PathBuf>> {
    let mut directories = vec![];
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            directories.push(path)
        }
    }
    Ok(directories)
}

fn read_package(directory: impl AsRef<Path>) -> anyhow::Result<Package> {
    let manifest_path = directory.as_ref().join("Cargo.toml");
    let metadata = cargo_metadata::MetadataCommand::new()
        .no_deps()
        .manifest_path(manifest_path)
        .exec()?;
    let package = metadata
        .packages
        .get(0)
        .ok_or_else(|| anyhow!("cannot retrieve package at {:?}", directory.as_ref()))?;
    Ok(package.clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore]
    fn one_crate_is_downloaded() {
        let crate_name = "rand";
        let crates = download_crates(&[crate_name]).unwrap();
        let rand = &crates[0];
        assert_eq!(rand.name, crate_name);
    }

    #[test]
    #[ignore]
    fn two_crates_are_downloaded() {
        let first_crate = "rand";
        let second_crate = "rust-gh-example";
        let crates = download_crates(&[first_crate, second_crate]).unwrap();
        assert_eq!(&crates[0].name, first_crate);
        assert_eq!(&crates[1].name, second_crate);
    }
}
