use std::{collections::BTreeMap, path::PathBuf};

use anyhow::Context;
use cargo_metadata::Package;
use tempfile::{tempdir, TempDir};

use crate::{download, next_ver};

pub struct PackagesCollection {
    packages: BTreeMap<String, Package>,
    /// Packages might be downloaded and stored in a temporary directory.
    /// It is stored here so that on drop it is deleted.
    _temp_dir: Option<TempDir>,
}

impl PackagesCollection {
    pub fn get_package(&self, package_name: &str) -> Option<&Package> {
        self.packages.get(package_name)
    }
}

pub fn get_registry_packages(
    registry_manifest: Option<&PathBuf>,
    local_packages: &[Package],
) -> anyhow::Result<PackagesCollection> {
    let (temp_dir, remote_packages) = match registry_manifest {
        Some(manifest) => (None, next_ver::public_packages(manifest)?),
        None => {
            let temp_dir = tempdir()?;
            let local_packages_names: Vec<&str> =
                local_packages.iter().map(|c| c.name.as_str()).collect();
            let directory = temp_dir.as_ref().to_str().context("invalid tempdir path")?;
            let registry_packages = download::download_packages(&local_packages_names, directory)?;
            (Some(temp_dir), registry_packages)
        }
    };
    let remote_packages = remote_packages
        .into_iter()
        .map(|c| {
            let package_name = c.name.clone();
            (package_name, c)
        })
        .collect();
    Ok(PackagesCollection {
        _temp_dir: temp_dir,
        packages: remote_packages,
    })
}
