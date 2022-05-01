use std::path::PathBuf;

use anyhow::Context;
use cargo_metadata::Package;
use crates_index::Index;
use git_cmd::Repo;
use tracing::{info, instrument};
use url::Url;

use crate::{
    cargo::{is_published, run_cargo, wait_until_published},
    publishable_packages,
    release_order::release_order,
};

#[derive(Debug)]
pub struct ReleaseRequest {
    /// The manifest of the project you want to release.
    pub local_manifest: PathBuf,
    /// Registry where you want to publish the packages.
    /// The registry name needs to be present in the Cargo config.
    /// If unspecified, the `publish` field of the package manifest is used.
    /// If the `publish` field is empty, crates.io is used.
    pub registry: Option<String>,
    /// Token used to publish to the cargo registry.
    pub token: Option<String>,
    /// Perform all checks without uploading.
    pub dry_run: bool,
}

/// Release the project as it is.
#[instrument]
pub async fn release(input: &ReleaseRequest) -> anyhow::Result<()> {
    let publishable_packages = publishable_packages(&input.local_manifest)?;
    let pkgs = &publishable_packages.iter().collect::<Vec<_>>();
    let release_order = release_order(pkgs);
    for package in release_order {
        let registry_indexes = registry_indexes(package, input.registry.clone())?;
        for mut index in registry_indexes {
            release_package(&mut index, package, input)?;
        }
    }
    Ok(())
}

/// Get the indexes where the package should be published.
/// If `registry` is specified, it takes precedence over the `publish` field
/// of the package manifest.
fn registry_indexes(package: &Package, registry: Option<String>) -> anyhow::Result<Vec<Index>> {
    let registries = registry
        .map(|r| vec![r])
        .unwrap_or_else(|| package.publish.clone().unwrap_or_default());
    let registry_urls = registries
        .iter()
        .map(|r| {
            cargo_edit::registry_url(package.manifest_path.as_ref(), Some(r))
                .context("failed to retrieve registry url")
        })
        .collect::<anyhow::Result<Vec<Url>>>()?;
    let mut registry_indexes = registry_urls
        .iter()
        .map(|u| Index::from_url(&format!("registry+{}", u)))
        .collect::<Result<Vec<Index>, crates_index::Error>>()?;
    if registry_indexes.is_empty() {
        registry_indexes.push(Index::new_cargo_default()?)
    }
    Ok(registry_indexes)
}

fn release_package(
    index: &mut Index,
    package: &Package,
    input: &ReleaseRequest,
) -> anyhow::Result<()> {
    if is_published(index, package)? {
        info!("{} {}: already published", package.name, package.version);
        return Ok(());
    }

    let mut args = vec!["publish"];
    args.push("--color");
    args.push("always");
    args.push("--manifest-path");
    args.push(package.manifest_path.as_ref());
    if let Some(token) = &input.token {
        args.push("--token");
        args.push(token);
    }
    if input.dry_run {
        args.push("--dry-run");
    }

    let workspace_root = input
        .local_manifest
        .parent()
        .expect("cannot find local_manifest parent");

    let repo = Repo::new(workspace_root)?;
    let (_, stderr) = run_cargo(workspace_root, &args)?;

    if !stderr.contains("Uploading") || stderr.contains("error:") {
        anyhow::bail!("failed to publish {}: {}", package.name, stderr);
    }

    if input.dry_run {
        info!(
            "{} {}: aborting upload due to dry run",
            package.name, package.version
        );
    } else {
        wait_until_published(index, package)?;

        let git_tag = format!("{}-v{}", package.name, package.version);
        repo.tag(&git_tag)?;
        repo.push(&git_tag)?;

        info!("published {}", package.name);
    }

    Ok(())
}
