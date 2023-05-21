use std::path::Path;

use cargo_lock::Lockfile;
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
    let local_lock = Lockfile::load(local_lock)?;
    let registry_lock = Lockfile::load(registry_lock)?;
    for local_package in local_lock.packages {
        // Cargo.lock can contain multiple packages with the same name but different versions.
        let registry_packages: Vec<&cargo_lock::Package> = registry_lock
            .packages
            .iter()
            .filter(|p| p.name == local_package.name)
            .collect();

        let is_same_version = registry_packages.is_empty()
            || registry_packages
                .iter()
                .any(|p| p.version == local_package.version);
        if !is_same_version {
            debug!(
                "Version of package {} changed to version {:?}",
                local_package.name, local_package.version
            );
            return Ok(true);
        }
    }
    Ok(false)
}
