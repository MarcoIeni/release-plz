use anyhow::Context;
use cargo_metadata::{Metadata, Package};
use std::path::Path;

pub fn get_manifest_metadata(manifest_path: &Path) -> anyhow::Result<cargo_metadata::Metadata> {
    cargo_metadata::MetadataCommand::new()
        .no_deps()
        .manifest_path(manifest_path)
        .exec()
        .with_context(|| format!("invalid manifest {manifest_path:?}"))
}

/// Lookup all members of the current workspace
pub fn workspace_members(metadata: &Metadata) -> anyhow::Result<impl Iterator<Item = Package>> {
    let workspace_members: std::collections::BTreeSet<_> =
        metadata.workspace_members.clone().into_iter().collect();
    let workspace_members = metadata
        .packages
        .clone()
        .into_iter()
        .filter(move |p| workspace_members.contains(&p.id))
        .map(|mut p| {
            p.manifest_path = canonicalize_path(p.manifest_path);
            for dep in p.dependencies.iter_mut() {
                dep.path = dep.path.take().map(canonicalize_path);
            }
            p
        });
    Ok(workspace_members)
}

fn canonicalize_path(
    path: cargo_metadata::camino::Utf8PathBuf,
) -> cargo_metadata::camino::Utf8PathBuf {
    if let Ok(path) = dunce::canonicalize(&path) {
        if let Ok(path) = path.try_into() {
            return path;
        }
    }

    path
}
