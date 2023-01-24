use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::anyhow;
use cargo_metadata::Package;

use crate::CHANGELOG_FILENAME;

pub trait PackagePath {
    fn package_path(&self) -> anyhow::Result<&Path>;

    fn changelog_path(&self) -> anyhow::Result<PathBuf> {
        let changelog_path = self.package_path()?.join(CHANGELOG_FILENAME);
        Ok(changelog_path)
    }

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
