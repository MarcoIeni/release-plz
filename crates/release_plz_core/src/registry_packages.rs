use std::{collections::BTreeMap, path::PathBuf};

use anyhow::Context;
use cargo_utils::LocalPackage;
use tempfile::{tempdir, TempDir};

use crate::{download, next_ver};

pub struct PackagesCollection {
    packages: BTreeMap<String, LocalPackage>,
    /// Packages might be downloaded and stored in a temporary directory.
    /// The directory is stored here so that it is deleted on drop
    _temp_dir: Option<TempDir>,
}

impl PackagesCollection {
    pub fn get_package(&self, package_name: &str) -> Option<&LocalPackage> {
        self.packages.get(package_name)
    }
}

pub fn get_registry_packages(
    registry_manifest: Option<&PathBuf>,
    local_packages: &[LocalPackage],
    registry: Option<&str>,
) -> anyhow::Result<PackagesCollection> {
    let (temp_dir, registry_packages) = match registry_manifest {
        Some(manifest) => (None, next_ver::publishable_packages(manifest)?),
        None => {
            let temp_dir = tempdir()?;
            let local_packages_names: Vec<&str> = local_packages
                .iter()
                .map(|c| c.package().name.as_str())
                .collect();
            let directory = temp_dir.as_ref().to_str().context("invalid tempdir path")?;
            let registry_packages =
                download::download_packages(&local_packages_names, directory, registry)?;
            (Some(temp_dir), registry_packages)
        }
    };
    let registry_packages = registry_packages
        .into_iter()
        .map(|c| {
            let package_name = c.name().to_string();
            (package_name, c)
        })
        .collect();
    Ok(PackagesCollection {
        _temp_dir: temp_dir,
        packages: registry_packages,
    })
}
