use std::path::Path;

use anyhow::Context;
use cargo_metadata::camino::Utf8Path;

pub fn strip_prefix(path: &Utf8Path, prefix: impl AsRef<Path>) -> anyhow::Result<&Utf8Path> {
    path.strip_prefix(prefix.as_ref())
        .with_context(|| format!("cannot strip prefix {:?} from {:?}", prefix.as_ref(), path))
}

pub fn to_utf8_path(path: &Path) -> anyhow::Result<&Utf8Path> {
    Utf8Path::from_path(path).with_context(|| format!("cannot convert {:?} to Utf8Path", path))
}
