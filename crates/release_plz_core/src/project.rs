use std::collections::{HashMap, HashSet};

use anyhow::Context as _;
use cargo_metadata::{
    camino::{Utf8Path, Utf8PathBuf},
    Metadata, Package,
};
use cargo_utils::CARGO_TOML;
use tracing::debug;

use crate::{
    copy_to_temp_dir, fs_utils::strip_prefix, manifest_dir, new_manifest_dir_path,
    root_repo_path_from_manifest_dir, tmp_repo::TempRepo, workspace_packages, Publishable as _,
    ReleaseMetadata, ReleaseMetadataBuilder,
};
use crate::{
    tera::{tera_context, tera_var, PACKAGE_VAR, VERSION_VAR},
    PackagePath as _,
};

#[derive(Debug)]
pub struct Project {
    /// Publishable packages.
    packages: Vec<Package>,
    /// Metadata for each release enabled package.
    release_metadata: HashMap<String, ReleaseMetadata>,
    /// Project root directory
    root: Utf8PathBuf,
    /// Directory containing the project manifest
    manifest_dir: Utf8PathBuf,
    /// The project contains more than one public package.
    /// Not affected by `single_package` option.
    contains_multiple_pub_packages: bool,
}

impl Project {
    pub fn new(
        local_manifest: &Utf8Path,
        single_package: Option<&str>,
        overrides: &HashSet<String>,
        metadata: &Metadata,
        release_metadata_builder: &dyn ReleaseMetadataBuilder,
    ) -> anyhow::Result<Self> {
        let manifest = local_manifest;
        let manifest_dir = manifest_dir(manifest)?.to_path_buf();
        debug!("manifest_dir: {manifest_dir:?}");
        let root = root_repo_path_from_manifest_dir(&manifest_dir)?;
        debug!("project_root: {root:?}");
        let mut packages = workspace_packages(metadata)?;
        check_overrides_typos(&packages, overrides)?;
        let mut release_metadata = HashMap::new();
        override_packages_path(&mut packages, metadata, &manifest_dir)
            .context("failed to override packages path")?;

        let packages_names: Vec<String> = packages.iter().map(|p| p.name.clone()).collect();
        packages.retain(|p| {
            let release_metadata =
                release_metadata_builder
                    .get_release_metadata(&p.name)
                    .map(|m| {
                        release_metadata.insert(p.name.clone(), m);
                    });
            release_metadata.is_some()
        });
        anyhow::ensure!(!packages.is_empty(), "no public packages found. Are there any public packages in your project? Analyzed packages: {packages_names:?}");

        let contains_multiple_pub_packages = packages.len() > 1;

        if let Some(pac) = single_package {
            packages.retain(|p| p.name == pac);
            anyhow::ensure!(
                !packages.is_empty(),
                "package `{}` not found. If it exists, is it public?",
                pac
            );
        }

        Ok(Self {
            packages,
            release_metadata,
            root,
            manifest_dir,
            contains_multiple_pub_packages,
        })
    }

    pub fn root(&self) -> &Utf8Path {
        &self.root
    }

    pub fn publishable_packages(&self) -> Vec<&Package> {
        self.packages
            .iter()
            .filter(|p| p.is_publishable())
            .collect()
    }

    /// Get all packages, including non-publishable.
    pub fn workspace_packages(&self) -> Vec<&Package> {
        self.packages.iter().collect()
    }

    /// Copy this project in a temporary repository and return the repository.
    /// We copy the project in another directory in order to avoid altering it.
    pub(crate) fn get_repo(&self) -> anyhow::Result<TempRepo> {
        let tmp_project_root_parent = copy_to_temp_dir(&self.root)?;
        let tmp_project_manifest_dir = new_manifest_dir_path(
            &self.root,
            &self.manifest_dir,
            tmp_project_root_parent.path(),
        )?;
        debug!("tmp_project_manifest_dir: {tmp_project_manifest_dir:?}");

        let tmp_project_root = new_project_root(&self.root, tmp_project_root_parent.path())?;
        let repository = TempRepo::new(tmp_project_root_parent, tmp_project_root)?;
        Ok(repository)
    }

    pub fn git_tag(&self, package_name: &str, version: &str) -> String {
        let mut tera = tera::Tera::default();
        let context = tera_context(package_name, version);

        let tag_template = self
            .release_metadata
            .get(package_name)
            .and_then(|m| m.tag_name_template.clone())
            .unwrap_or({
                if self.contains_multiple_pub_packages {
                    format!("{}-v{}", tera_var(PACKAGE_VAR), tera_var(VERSION_VAR))
                } else {
                    format!("v{}", tera_var(VERSION_VAR))
                }
            });

        crate::tera::render_template(&mut tera, &tag_template, &context, "tag_name")
    }

    pub fn release_name(&self, package_name: &str, version: &str) -> String {
        let mut tera = tera::Tera::default();
        let context = tera_context(package_name, version);

        let name_template = self
            .release_metadata
            .get(package_name)
            .and_then(|m| m.release_name_template.clone())
            .unwrap_or({
                if self.contains_multiple_pub_packages {
                    format!("{}-v{}", tera_var(PACKAGE_VAR), tera_var(VERSION_VAR))
                } else {
                    format!("v{}", tera_var(VERSION_VAR))
                }
            });

        crate::tera::render_template(&mut tera, &name_template, &context, "release_name")
    }

    pub fn cargo_lock_path(&self) -> Utf8PathBuf {
        self.root.join("Cargo.lock")
    }
}

fn check_overrides_typos(
    packages: &[Package],
    overrides: &HashSet<String>,
) -> Result<(), anyhow::Error> {
    let package_names: HashSet<_> = packages.iter().map(|p| p.name.clone()).collect();
    check_for_typos(&package_names, overrides)?;
    Ok(())
}

/// Check for typos in the package names based on the overrides
fn check_for_typos(packages: &HashSet<String>, overrides: &HashSet<String>) -> anyhow::Result<()> {
    let diff: Vec<_> = overrides.difference(packages).collect();

    if diff.is_empty() {
        Ok(())
    } else {
        let mut missing: Vec<_> = diff.into_iter().collect();
        missing.sort();
        let missing = missing
            .iter()
            .map(|s| format!("`{s}`"))
            .collect::<Vec<_>>()
            .join(", ");

        Err(anyhow::anyhow!(
            "The following overrides are not present in the workspace: {missing}. Check for typos"
        ))
    }
}

pub fn new_project_root(
    original_project_root: &Utf8Path,
    new_project_root_parent: &Utf8Path,
) -> anyhow::Result<Utf8PathBuf> {
    let project_root_dirname = original_project_root
        .file_name()
        .context("cannot get project root dirname")?;
    Ok(new_project_root_parent.join(project_root_dirname))
}

/// Cargo metadata contains package paths of the original user project.
/// Release-plz copies the user project to a temporary
/// directory to avoid making changes to the original project.
/// This function sets packages path relative to the specified `manifest_dir`.
fn override_packages_path(
    packages: &mut Vec<Package>,
    metadata: &Metadata,
    manifest_dir: &Utf8Path,
) -> Result<(), anyhow::Error> {
    let canonicalized_workspace_root =
        dunce::canonicalize(&metadata.workspace_root).with_context(|| {
            format!(
                "failed to canonicalize workspace root {:?}",
                metadata.workspace_root
            )
        })?;
    for p in packages {
        let old_path = p.package_path()?;
        let relative_package_path =
            strip_prefix(old_path, &canonicalized_workspace_root)?.to_path_buf();
        p.manifest_path = manifest_dir.join(relative_package_path).join(CARGO_TOML);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ReleaseMetadataBuilder;
    use cargo_utils::get_manifest_metadata;

    struct ReleaseMetadataBuilderStub {
        release: bool,
        tag_name: Option<String>,
        release_name: Option<String>,
    }

    impl ReleaseMetadataBuilderStub {
        pub fn new(release: bool, tag_name: Option<String>, release_name: Option<String>) -> Self {
            Self {
                release,
                tag_name,
                release_name,
            }
        }
    }

    impl ReleaseMetadataBuilder for ReleaseMetadataBuilderStub {
        fn get_release_metadata(&self, _package_name: &str) -> Option<ReleaseMetadata> {
            self.release.then(|| ReleaseMetadata {
                tag_name_template: self.tag_name.clone(),
                release_name_template: self.release_name.clone(),
            })
        }
    }

    fn get_project(
        local_manifest: &Utf8Path,
        single_package: Option<&str>,
        overrides: &HashSet<String>,
        is_release_enabled: bool,
        tag_name: Option<String>,
        release_name: Option<String>,
    ) -> anyhow::Result<Project> {
        let metadata = get_manifest_metadata(local_manifest).unwrap();
        let release_metadata_builder =
            ReleaseMetadataBuilderStub::new(is_release_enabled, tag_name, release_name);
        Project::new(
            local_manifest,
            single_package,
            overrides,
            &metadata,
            &release_metadata_builder,
        )
    }

    #[test]
    fn test_for_typos() {
        let packages: HashSet<String> = vec!["foo".to_string()].into_iter().collect();
        let overrides: HashSet<String> = vec!["bar".to_string()].into_iter().collect();
        let result = check_for_typos(&packages, &overrides);
        assert_eq!(
            result.unwrap_err().to_string(),
            "The following overrides are not present in the workspace: `bar`. Check for typos"
        );
    }

    #[test]
    fn test_empty_override() {
        let utf8_path = Utf8Path::new("../../tests/fixtures/typo-in-overrides/Cargo.toml");
        let local_manifest = utf8_path;
        let result = get_project(local_manifest, None, &HashSet::default(), true, None, None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_successful_override() {
        let local_manifest = Utf8Path::new("../../tests/fixtures/typo-in-overrides/Cargo.toml");
        let overrides = (["typo_test".to_string()]).into();
        let result = get_project(local_manifest, None, &overrides, true, None, None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_typo_in_crate_names() {
        let local_manifest = Utf8Path::new("../../tests/fixtures/typo-in-overrides/Cargo.toml");
        let single_package = None;
        let overrides = vec!["typo_tesst".to_string()].into_iter().collect();
        let result = get_project(local_manifest, single_package, &overrides, true, None, None);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "The following overrides are not present in the workspace: `typo_tesst`. Check for typos"
        );
    }

    #[test]
    fn project_new_no_release_will_error() {
        let local_manifest = Utf8Path::new("../fake_package/Cargo.toml");
        let result = get_project(local_manifest, None, &HashSet::default(), false, None, None);
        assert!(result.is_err());
        expect_test::expect![[r#"no public packages found. Are there any public packages in your project? Analyzed packages: ["cargo_utils", "fake_package", "git_cmd", "test_logs", "next_version", "release-plz", "release_plz_core"]"#]]
        .assert_eq(&result.unwrap_err().to_string());
    }

    #[test]
    fn project_tag_template_none() {
        let local_manifest = Utf8Path::new("../../tests/fixtures/typo-in-overrides/Cargo.toml");
        let project = get_project(local_manifest, None, &HashSet::default(), true, None, None)
            .expect("Should ok");
        assert_eq!(project.git_tag("typo_test", "0.1.0"), "v0.1.0");
    }

    #[test]
    fn project_release_and_tag_template_some() {
        let local_manifest = Utf8Path::new("../../tests/fixtures/typo-in-overrides/Cargo.toml");
        let project = get_project(
            local_manifest,
            None,
            &HashSet::default(),
            true,
            Some("prefix-{{ package }}-middle-{{ version }}-postfix".to_string()),
            Some("release-prefix-{{ package }}-middle-{{ version }}-postfix".to_string()),
        )
        .expect("Should ok");
        assert_eq!(
            project.git_tag("typo_test", "0.1.0"),
            "prefix-typo_test-middle-0.1.0-postfix"
        );
        assert_eq!(
            project.release_name("typo_test", "0.1.0"),
            "release-prefix-typo_test-middle-0.1.0-postfix"
        );
    }
}
