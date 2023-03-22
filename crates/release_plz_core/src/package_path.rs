use std::{fs, path::PathBuf};

use crate::CHANGELOG_FILENAME;

pub trait PackagePath {
    fn changelog_path(&self) -> anyhow::Result<PathBuf> {
        let changelog_path = self.package_path()?.join(CHANGELOG_FILENAME);
        Ok(changelog_path)
    }
}
