//! Download packages from cargo registry, similar to the `git clone` behavior.

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
pub fn download_packages(packages: &[&str]) -> anyhow::Result<Vec<Package>> {
    let config = cargo::Config::default().expect("Unable to get cargo config.");
    let source_id = SourceId::crates_io(&config).expect("Unable to retrieve source id.");
    let packages: Vec<cargo_clone::Crate> = packages
        .iter()
        .map(|c| cargo_clone::Crate::new(c.to_string(), None))
        .collect();
    let temp_dir = tempdir()?;
    let directory = temp_dir.as_ref().to_str().expect("invalid path");
    let clone_opts = cargo_clone::CloneOpts::new(&packages, &source_id, Some(directory), false);
    cargo_clone::clone(&clone_opts, &config).context("cannot download remote packages")?;
    let packages = if packages.len() == 1 {
        vec![read_package(directory)?]
    } else {
        let packages = sub_directories(directory)?;
        let packages = packages
            .iter()
            .map(|p| read_package(&p))
            .collect::<Result<Vec<Package>, _>>();
        packages?
    };
    Ok(packages)
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
    fn one_package_is_downloaded() {
        let package_name = "rand";
        let packages = download_packages(&[package_name]).unwrap();
        let rand = &packages[0];
        assert_eq!(rand.name, package_name);
    }

    #[test]
    #[ignore]
    fn two_packages_are_downloaded() {
        let first_package = "rand";
        let second_package = "rust-gh-example";
        let packages = download_packages(&[first_package, second_package]).unwrap();
        assert_eq!(&packages[0].name, first_package);
        assert_eq!(&packages[1].name, second_package);
    }
}
