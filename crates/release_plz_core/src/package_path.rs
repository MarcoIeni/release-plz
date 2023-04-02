use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::anyhow;
use cargo_metadata::Package;

pub trait PackagePath {
    fn package_path(&self) -> anyhow::Result<&Path>;

    fn canonical_path(&self) -> anyhow::Result<PathBuf> {
        let p = fs::canonicalize(self.package_path()?)?;
        Ok(p)
    }
}

impl PackagePath for Package {
    fn package_path(&self) -> anyhow::Result<&Path> {
        manifest_dir(self.manifest_path.as_std_path())
    }
}

pub fn manifest_dir(manifest: &Path) -> anyhow::Result<&Path> {
    let manifest_dir = manifest.parent().ok_or_else(|| {
        anyhow!(
            "Cannot find directory where manifest {:?} is located",
            manifest
        )
    })?;
    Ok(manifest_dir)
}
