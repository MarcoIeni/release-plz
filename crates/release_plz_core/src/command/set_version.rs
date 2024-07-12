use std::collections::BTreeMap;

use anyhow::Context;
use cargo_metadata::{
    camino::{Utf8Path, Utf8PathBuf},
    semver::Version,
    Metadata, Package,
};
use cargo_utils::{canonical_local_manifest, workspace_members, LocalManifest};

use crate::{changelog_parser::last_release_from_str, PackagePath as _, CHANGELOG_FILENAME};

pub struct SetVersionRequest {
    /// The manifest of the project you want to update.
    manifest: Utf8PathBuf,
    /// Cargo metadata.
    metadata: Metadata,
    /// <package name, version change>
    version_changes: BTreeMap<String, VersionChange>,
}

impl SetVersionRequest {
    pub fn set_changelog_path(&mut self, package: &str, changelog_path: Utf8PathBuf) {
        self.version_changes
            .entry(package.to_string())
            .and_modify(|change| {
                change.with_changelog_path(changelog_path);
            });
    }
}

pub struct VersionChange {
    version: Version,
    /// This path needs to be a relative path to the Cargo.toml of the project.
    /// I.e. if you have a workspace, it needs to be relative to the workspace root.
    pub changelog_path: Option<Utf8PathBuf>,
}

impl VersionChange {
    pub fn new(version: Version) -> Self {
        Self {
            version,
            changelog_path: None,
        }
    }

    pub fn with_changelog_path(&mut self, changelog_path: Utf8PathBuf) {
        self.changelog_path = Some(changelog_path);
    }
}

impl SetVersionRequest {
    pub fn new(
        version_changes: BTreeMap<String, VersionChange>,
        metadata: Metadata,
    ) -> anyhow::Result<Self> {
        let manifest = cargo_utils::workspace_manifest(&metadata);
        let manifest = canonical_local_manifest(manifest.as_ref())?;
        Ok(Self {
            version_changes,
            metadata,
            manifest,
        })
    }
}

pub fn set_version(input: &SetVersionRequest) -> anyhow::Result<()> {
    let workspace_manifest = LocalManifest::try_new(&input.manifest)?;
    let packages: BTreeMap<String, Package> = workspace_members(&input.metadata)?
        .map(|p| {
            let package_name = p.name.clone();
            (package_name, p)
        })
        .collect();
    let all_packages: Vec<&Package> = packages.values().collect();
    for (package, change) in &input.version_changes {
        let pkg = packages
            .get(package)
            .with_context(|| format!("package {package} not found"))?;
        let pkg_path = pkg.package_path()?;
        super::update::set_version(
            &all_packages,
            pkg_path,
            &change.version,
            &workspace_manifest.path,
        )?;

        let default_changelog_path = pkg_path.join(CHANGELOG_FILENAME);
        let changelog_path: &Utf8Path = change
            .changelog_path
            .as_deref()
            .unwrap_or(&default_changelog_path);
        update_changelog(changelog_path, &pkg.version, &change.version)
            .with_context(|| format!("failed to update changelog at {changelog_path}"))?;
    }
    Ok(())
}

fn update_changelog(
    changelog_path: &Utf8Path,
    old_version: &Version,
    new_version: &Version,
) -> anyhow::Result<()> {
    let changelog_content = fs_err::read_to_string(changelog_path)?;
    let last_release = last_release_from_str(&changelog_content)?.context("no release found")?;

    let new_changelog_content = {
        let old_title = last_release.title();
        // replace the new version. `replacen` doesn't work, because we
        // also want to replace the version in the release link.
        let new_title = old_title.replace(&old_version.to_string(), &new_version.to_string());
        changelog_content.replacen(old_title, &new_title, 1)
    };

    fs_err::write(changelog_path, new_changelog_content)?;

    Ok(())
}
