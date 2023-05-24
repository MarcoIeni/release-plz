use std::{collections::HashMap, path::Path};

use anyhow::Context;
use serde::Deserialize;
use tracing::debug;

/// Compare the dependencies present in the `Cargo.lock` of the registry package and the local one.
/// Check if the dependencies of the registry package were updated.
/// This method doesn't detect if the local Cargo.lock added new packages: just if
/// the version of the packages changed.
/// This is enough to understand if the package was updated.
pub fn are_lock_dependencies_updated(
    local_lock: &Path,
    registry_package: &Path,
) -> anyhow::Result<bool> {
    let registry_lock = &registry_package.join("Cargo.lock");
    if !local_lock.exists() || !registry_lock.exists() {
        return Ok(false);
    }
    are_dependencies_updated(local_lock, registry_lock)
}

fn are_dependencies_updated(local_lock: &Path, registry_lock: &Path) -> anyhow::Result<bool> {
    let local_lock: Lockfile = read_lockfile(local_lock)
        .with_context(|| format!("failed to load lockfile of local package {:?}", local_lock))?;
    let registry_lock = read_lockfile(registry_lock).with_context(|| {
        format!(
            "failed to load lockfile of registry package {:?}",
            registry_lock
        )
    })?;
    let registry_lock_packages = PackagesByName::new(&registry_lock.packages);
    Ok(are_dependencies_of_lockfiles_updated(
        &local_lock,
        &registry_lock_packages,
    ))
}

fn read_lockfile(path: &Path) -> anyhow::Result<Lockfile> {
    let content =
        std::fs::read_to_string(path).with_context(|| format!("can't read lockfile {path:?}"))?;
    let lockfile =
        toml::from_str(&content).with_context(|| format!("invalid format of lockfile {path:?}"))?;
    Ok(lockfile)
}

fn are_dependencies_of_lockfiles_updated(
    local_lock: &Lockfile,
    registry_lock: &PackagesByName,
) -> bool {
    for local_package in &local_lock.packages {
        if let Some(registry_packages) = registry_lock.get(&local_package.name) {
            let is_version_present = registry_packages
                .iter()
                .any(|p| p.version == local_package.version);
            if !is_version_present {
                debug!(
                    "Version of package {} changed to version {:?}",
                    local_package.name, local_package.version
                );
                return true;
            }
        }
    }
    false
}

#[derive(Deserialize, Debug)]
struct Lockfile {
    packages: Vec<Package>,
}

#[derive(Deserialize, Debug)]
struct Package {
    name: String,
    version: String,
}

/// Packages grouped by name, to search faster.
/// Cargo.lock can contain multiple packages with the same name but different versions.
struct PackagesByName<'a> {
    packages: HashMap<&'a str, Vec<&'a Package>>,
}

impl<'a> PackagesByName<'a> {
    fn new(packages: &'a [Package]) -> Self {
        let mut packages_by_name = HashMap::new();
        for package in packages {
            packages_by_name
                .entry(package.name.as_str())
                .or_insert_with(Vec::new)
                .push(package);
        }
        Self {
            packages: packages_by_name,
        }
    }

    /// Get the packages with the given name.
    fn get(&self, name: &str) -> Option<&[&Package]> {
        self.packages.get(name).map(|p| {
            // If the entry exists, it contains at least one package.
            assert!(!p.is_empty());
            p.as_slice()
        })
    }
}
