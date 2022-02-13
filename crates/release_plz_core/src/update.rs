use crate::{tmp_repo::TempRepo, PackagePath, UpdateRequest};
use cargo_edit::LocalManifest;
use cargo_metadata::{Package, Version};
use std::path::Path;

use tracing::{debug, instrument};

/// Update a local rust project
#[instrument]
pub fn update(input: &UpdateRequest) -> anyhow::Result<(Vec<(Package, Version)>, TempRepo)> {
    let (packages_to_update, repository) = crate::next_versions(input)?;
    update_versions(&packages_to_update)?;
    Ok((packages_to_update, repository))
}

#[instrument(skip_all)]
fn update_versions(local_packages: &[(Package, Version)]) -> anyhow::Result<()> {
    for (package, next_version) in local_packages {
        let package_path = package.package_path()?;
        set_version(package_path, next_version);
    }
    Ok(())
}

#[instrument]
fn set_version(package_path: &Path, version: &Version) {
    debug!("updating version");
    let mut local_manifest =
        LocalManifest::try_new(&package_path.join("Cargo.toml")).expect("cannot read manifest");
    local_manifest.set_package_version(version);
    local_manifest.write().expect("cannot update manifest");
}
