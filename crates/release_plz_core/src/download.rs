//! Download packages from cargo registry, similar to the `git clone` behavior.

use std::{
    fmt, fs,
    path::{Path, PathBuf},
};

use anyhow::{anyhow, Context};
use cargo::core::SourceId;
use cargo_metadata::Package;
use tracing::{info, instrument};

use crate::CARGO_TOML;

#[instrument]
pub fn download_packages(
    packages: &[&str],
    directory: impl AsRef<str> + fmt::Debug,
    registry: Option<&str>,
) -> anyhow::Result<Vec<Package>> {
    let directory = directory.as_ref();
    info!("downloading packages from cargo registry");
    let config = cargo::Config::default().expect("Unable to get cargo config.");
    let source_id = match registry {
        Some(registry) => SourceId::alt_registry(&config, registry)
            .with_context(|| format!("Unable to retrieve source id for registry {registry}")),
        None => SourceId::crates_io(&config).context("Unable to retrieve source id for crates.io."),
    }?;
    let packages: Vec<cargo_clone::Crate> = packages
        .iter()
        .map(|c| cargo_clone::Crate::new(c.to_string(), None))
        .collect();
    let clone_opts = cargo_clone::CloneOpts::new(&packages, &source_id, Some(directory), false);
    cargo_clone::clone(&clone_opts, &config).context("cannot download packages from registry")?;
    let packages = match packages.len() {
        1 => vec![read_package(directory)?],
        _ => {
            let packages = sub_directories(directory)?;
            let packages = packages
                .iter()
                .map(|p| read_package(&p))
                .collect::<Result<Vec<Package>, _>>();
            packages?
        }
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

/// Read a package from file system
pub fn read_package(directory: impl AsRef<Path>) -> anyhow::Result<Package> {
    let manifest_path = directory.as_ref().join(CARGO_TOML);
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
    use tempfile::tempdir;

    use super::*;

    #[test]
    #[ignore]
    fn one_package_is_downloaded() {
        let package_name = "rand";
        let temp_dir = tempdir().unwrap();
        let directory = temp_dir.as_ref().to_str().expect("invalid tempdir path");
        let packages = download_packages(&[package_name], directory, None).unwrap();
        let rand = &packages[0];
        assert_eq!(rand.name, package_name);
    }

    #[test]
    #[ignore]
    fn two_packages_are_downloaded() {
        let first_package = "rand";
        let second_package = "rust-gh-example";
        let temp_dir = tempdir().unwrap();
        let directory = temp_dir.as_ref().to_str().expect("invalid tempdir path");
        let packages =
            download_packages(&[first_package, second_package], directory, None).unwrap();
        assert_eq!(&packages[0].name, first_package);
        assert_eq!(&packages[1].name, second_package);
    }
}
