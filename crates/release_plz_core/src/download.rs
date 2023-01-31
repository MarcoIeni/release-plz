//! Download packages from cargo registry, similar to the `git clone` behavior.

use std::{fmt, path::Path};

use anyhow::{anyhow, Context};
use cargo_metadata::Package;
use tracing::{info, instrument, warn};

use crate::{
    clone::{Cloner, ClonerSource, Crate},
    CARGO_TOML,
};

#[instrument]
pub fn download_packages(
    packages: &[&str],
    directory: impl AsRef<str> + fmt::Debug,
    registry: Option<&str>,
) -> anyhow::Result<Vec<Package>> {
    let directory = directory.as_ref();
    info!("downloading packages from cargo registry");
    let source: ClonerSource = match registry {
        Some(registry) => ClonerSource::registry(registry),
        None => ClonerSource::crates_io(),
    };
    let crates: Vec<Crate> = packages
        .iter()
        .map(|&package_name| Crate::new(package_name.to_string(), None))
        .collect();
    let downloaded_packages = Cloner::builder()
        .with_directory(directory)
        .with_source(source)
        .build()
        .context("can't build cloner")?
        .clone(&crates)
        .context("error while downloading packages")?;

    let pkgs = downloaded_packages
        .iter()
        .map(|package| {
            let dir_path: &Path = directory.as_ref();
            let package_path = dir_path.join(package.name());
            (package.name(), package_path)
        })
        .filter_map(|(package_name, package_path)| {
            read_package(package_path)
                .map_err(|e| warn!("can't download {}: {:?}", package_name, e))
                // Filter non-existing packages.
                // Unfortunately we filter also packages that we couldn't download due to network issues.
                .ok()
        })
        .collect();
    Ok(pkgs)
}

/// Read a package from file system
pub fn read_package(directory: impl AsRef<Path>) -> anyhow::Result<Package> {
    let manifest_path = directory.as_ref().join(CARGO_TOML);
    let metadata = cargo_metadata::MetadataCommand::new()
        .no_deps()
        .manifest_path(manifest_path)
        .exec()
        .context("failed to execute cargo_metadata")?;
    let package = metadata
        .packages
        .get(0)
        .ok_or_else(|| anyhow!("cannot retrieve package at {:?}", directory.as_ref()))?;
    Ok(package.clone())
}

#[cfg(test)]
mod tests {
    use fake::Fake;
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

    #[test]
    #[ignore]
    fn downloading_non_existing_package_does_not_error() {
        // Generate random string 15 characters long.
        let package: String = 15.fake();
        let temp_dir = tempdir().unwrap();
        let directory = temp_dir.as_ref().to_str().expect("invalid tempdir path");
        download_packages(&[&package], directory, None).unwrap();
    }
}
