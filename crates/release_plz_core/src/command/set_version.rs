use std::collections::BTreeMap;

use anyhow::Context;
use cargo_metadata::{camino::Utf8PathBuf, semver::Version, Metadata, Package};
use cargo_utils::{canonical_local_manifest, workspace_members, LocalManifest};

use crate::PackagePath as _;

pub struct SetVersionRequest {
    /// The manifest of the project you want to update.
    manifest: Utf8PathBuf,
    /// Cargo metadata.
    metadata: Metadata,
    version_changes: Vec<VersionChange>,
}

pub struct VersionChange {
    package: String,
    version: Version,
}

impl VersionChange {
    pub fn new(package: String, version: Version) -> Self {
        Self { package, version }
    }
}

impl SetVersionRequest {
    pub fn new(version_changes: Vec<VersionChange>, metadata: Metadata) -> anyhow::Result<Self> {
        let manifest = cargo_utils::workspace_manifest(&metadata);
        let manifest = canonical_local_manifest(manifest.as_ref())?;
        Ok(Self {
            version_changes,
            metadata,
            manifest,
        })
    }
}

pub fn set_version(input: &SetVersionRequest) -> anyhow::Result<()> {
    let mut workspace_manifest = LocalManifest::try_new(&input.manifest)?;
    let packages: BTreeMap<String, Package> = workspace_members(&input.metadata)?
        .map(|p| {
            let package_name = p.name.clone();
            (package_name, p)
        })
        .collect();
    // TODO: ref
    let all_packages: Vec<&Package> = packages.values().collect();
    for change in &input.version_changes {
        let pkg = packages
            .get(&change.package)
            .with_context(|| format!("package {} not found", &change.package))?;
        super::update::set_version(
            &all_packages,
            pkg.package_path()?,
            &change.version,
            workspace_manifest,
        );
    }
    Ok(())
}
