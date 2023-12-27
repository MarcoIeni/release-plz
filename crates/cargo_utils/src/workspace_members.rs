use anyhow::Context;
use cargo_metadata::Package;
use std::path::Path;

/// Lookup all members of the current workspace
pub fn workspace_members(
    manifest_path: Option<&Path>,
) -> anyhow::Result<impl Iterator<Item = Package>> {
    let mut cmd = cargo_metadata::MetadataCommand::new();
    cmd.no_deps();
    if let Some(manifest_path) = manifest_path {
        cmd.manifest_path(manifest_path);
    }
    let result = cmd.exec().with_context(|| "Invalid manifest")?;
    let workspace_members: std::collections::BTreeSet<_> =
        result.workspace_members.into_iter().collect();
    let workspace_members = result
        .packages
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
