use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::anyhow;
use cargo_metadata::Package;

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

impl PackagePath for Package {}
