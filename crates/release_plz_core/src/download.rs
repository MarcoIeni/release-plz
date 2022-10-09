//! Download packages from cargo registry, similar to the `git clone` behavior.

use std::{fmt, path::Path};

use anyhow::{anyhow, Context};
use cargo_clone_core::SourceId;
use cargo_metadata::Package;
use tracing::{info, instrument, warn};

use crate::CARGO_TOML;

#[instrument]
pub fn download_packages(
    packages: &[&str],
    directory: impl AsRef<str> + fmt::Debug,
    registry: Option<&str>,
) -> anyhow::Result<Vec<Package>> {
    let directory = directory.as_ref();
    info!("downloading packages from cargo registry");
    let config = cargo_clone_core::Config::default().expect("Unable to get cargo config.");
    let source_id = match registry {
        Some(registry) => SourceId::alt_registry(&config, registry)
            .with_context(|| format!("Unable to retrieve source id for registry {registry}")),
        None => SourceId::crates_io(&config).context("Unable to retrieve source id for crates.io."),
    }?;
    packages
        .iter()
        .map(|&package_name| {
            (
                package_name,
                cargo_clone_core::Crate::new(package_name.to_string(), None),
            )
        })
        .filter_map(|(package_name, package)| {
            let packages = &[package];
            let dir_path: &Path = directory.as_ref();
            let package_path = dir_path.join(package_name);
            let package_path = package_path
                .as_path()
                .as_os_str()
                .to_str()
                .expect("can't convert os string into string");
            let clone_opts =
                cargo_clone_core::CloneOpts::new(packages, &source_id, Some(package_path), false);
            // Filter non-existing packages.
            // Unfortunately, we also filters packages we couldn't
            // download due to other issues, such as network.
            cargo_clone_core::clone(&clone_opts, &config)
                .map(|()| (read_package(package_path)))
                .map_err(|e| warn!("can't download {}: {}", package_name, e))
                .ok()
        })
        .collect()
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
    use claim::assert_ok;
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
        assert_ok!(download_packages(&[&package], directory, None));
    }
}
