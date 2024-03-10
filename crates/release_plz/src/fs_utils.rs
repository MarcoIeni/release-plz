use std::path::{Path, PathBuf};

use anyhow::Context;
use cargo_metadata::camino::{Utf8Path, Utf8PathBuf};

pub fn to_utf8_path(path: &Path) -> anyhow::Result<&Utf8Path> {
    Utf8Path::from_path(path).with_context(|| format!("cannot convert {:?} to Utf8Path", path))
}

pub fn to_utf8_pathbuf(path: PathBuf) -> anyhow::Result<Utf8PathBuf> {
    match Utf8PathBuf::from_path_buf(path) {
        Ok(p) => Ok(p),
        Err(path) => Err(anyhow::anyhow!("cannot convert {:?} to Utf8Path", &path)),
    }
}
