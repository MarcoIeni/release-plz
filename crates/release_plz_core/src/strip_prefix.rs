use std::path::Path;

use anyhow::Context;

pub fn strip_prefix(path: &Path, prefix: impl AsRef<Path>) -> anyhow::Result<&Path> {
    path.strip_prefix(prefix.as_ref())
        .with_context(|| format!("cannot strip prefix {:?} from {:?}", prefix.as_ref(), path))
}
