use std::path::{Path, PathBuf};

use anyhow::Context;
use cargo_metadata::Package;
use semver::Version;

use crate::LocalManifest;

#[derive(Debug)]
pub struct LocalPackage {
    package: Package,
    manifest: LocalManifest,
}

impl LocalPackage {
    pub fn new(package: Package) -> anyhow::Result<Self> {
        let manifest_path = package.manifest_path.as_ref();
        let manifest = LocalManifest::try_new(manifest_path)
            .with_context(|| format!("can't handle local manifest {manifest_path:?}"))?;
        Ok(Self { package, manifest })
    }

    pub fn package(&self) -> &Package {
        &self.package
    }

    pub fn name(&self) -> &str {
        &self.package.name
    }

    pub fn version(&self) -> &Version {
        &self.package.version
    }

    pub fn manifest(&self) -> &LocalManifest {
        &self.manifest
    }

    pub fn is_library(&self) -> bool {
        self.package
            .targets
            .iter()
            .any(|t| t.kind.contains(&"lib".to_string()))
    }

    pub fn package_path(&self) -> &Path {
        manifest_dir(&self.manifest.path).unwrap()
    }
}

fn manifest_dir(manifest: &Path) -> anyhow::Result<&Path> {
    let manifest_dir = manifest.parent().with_context(|| {
        format!(
            "Cannot find directory where manifest {:?} is located",
            manifest
        )
    })?;
    Ok(manifest_dir)
}
