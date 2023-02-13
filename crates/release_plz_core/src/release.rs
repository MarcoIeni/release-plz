use std::path::{Path, PathBuf};

use anyhow::Context;
use cargo_metadata::Package;
use crates_index::Index;
use git_cmd::Repo;
use secrecy::{ExposeSecret, SecretString};
use tracing::{info, instrument, warn};
use url::Url;

use crate::{
    backend::GitClient,
    cargo::{is_published, run_cargo, wait_until_published},
    changelog_parser,
    release_order::release_order,
    GitBackend, PackagePath, Project, RepoUrl,
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
    pub token: Option<SecretString>,
    /// Perform all checks without uploading.
    pub dry_run: bool,
    /// Publishes GitHub release.
    pub git_release: Option<GitRelease>,
    /// GitHub repo URL.
    pub repo_url: Option<String>,
    /// Don't verify the contents by building them.
    /// If true, `release-plz` adds the `--no-verify` flag to `cargo publish`.
    pub no_verify: bool,
    /// Allow dirty working directories to be packaged.
    /// If true, `release-plz` adds the `--allow-dirty` flag to `cargo publish`.
    pub allow_dirty: bool,
}

#[derive(Debug)]
pub struct GitRelease {
    /// Git token used to publish release.
    pub git_token: SecretString,
    ///Kind of Git Backend.
    pub backend: GitBackend,
}

impl ReleaseRequest {
    fn workspace_root(&self) -> anyhow::Result<&Path> {
        crate::manifest_dir(&self.local_manifest).context("cannot find local_manifest parent")
    }
}

/// Release the project as it is.
#[instrument]
pub async fn release(input: &ReleaseRequest) -> anyhow::Result<()> {
    let project = Project::new(&input.local_manifest, None)?;
    let pkgs = project.packages().iter().collect::<Vec<_>>();
    let release_order = release_order(&pkgs);
    for package in release_order {
        let workspace_root = input.workspace_root()?;
        let repo = Repo::new(workspace_root)?;
        let git_tag = project.git_tag(&package.name, &package.version.to_string());
        if repo.tag_exists(&git_tag)? {
            info!(
                "{} {}: Already published - Tag {} already exists",
                package.name, package.version, &git_tag
            );
            continue;
        }
        let registry_indexes = registry_indexes(package, input.registry.clone())?;
        for mut index in registry_indexes {
            if is_published(&mut index, package)? {
                info!("{} {}: already published", package.name, package.version);
                return Ok(());
            }
            release_package(&mut index, package, input, git_tag.clone()).await?;
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
            cargo_utils::registry_url(package.manifest_path.as_ref(), Some(r))
                .context("failed to retrieve registry url")
        })
        .collect::<anyhow::Result<Vec<Url>>>()?;
    let mut registry_indexes = registry_urls
        .iter()
        .map(|u| Index::from_url(&format!("registry+{u}")))
        .collect::<Result<Vec<Index>, crates_index::Error>>()?;
    if registry_indexes.is_empty() {
        registry_indexes.push(Index::new_cargo_default()?)
    }
    Ok(registry_indexes)
}

async fn release_package(
    index: &mut Index,
    package: &Package,
    input: &ReleaseRequest,
    git_tag: String,
) -> anyhow::Result<()> {
    let mut args = vec!["publish"];
    args.push("--color");
    args.push("always");
    args.push("--manifest-path");
    args.push(package.manifest_path.as_ref());
    if let Some(token) = &input.token {
        args.push("--token");
        args.push(token.expose_secret());
    }
    if input.dry_run {
        args.push("--dry-run");
    }
    if input.allow_dirty {
        args.push("--allow-dirty");
    }
    if input.no_verify {
        args.push("--no-verify");
    }
    let workspace_root = input.workspace_root()?;

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

        repo.tag(&git_tag)?;
        repo.push(&git_tag)?;

        info!("published {} {}", package.name, package.version);

        if let Some(git_release) = &input.git_release {
            let release_body = release_body(package);
            let backend = input.git_release.unwrap().backend;
            publish_release(
                git_tag,
                input.repo_url.as_deref(),
                repo,
                &release_body,
                git_release.git_token.clone(),
                backend,
            )
            .await?;
        }
    }

    Ok(())
}

/// Return an empty string if the changelog cannot be parsed.
fn release_body(package: &Package) -> String {
    let changelog_path = package.changelog_path().unwrap();
    match changelog_parser::last_changes(&changelog_path) {
        Ok(Some(changes)) => changes,
        Ok(None) => {
            warn!(
                "{}: last change not fuond in changelog at path {:?}. The git release body will be empty.",
                package.name, &changelog_path
            );
            String::new()
        }
        Err(e) => {
            warn!(
                "{}: failed to parse changelog at path {:?}: {}. The git release body will be empty.",
                package.name, &changelog_path, e
            );
            String::new()
        }
    }
}

async fn publish_release(
    git_tag: String,
    repo_url: Option<&str>,
    repo: Repo,
    release_body: &str,
    git_token: SecretString,
    backend: GitBackend,
) -> anyhow::Result<()> {
    let repo_url = match repo_url {
        Some(url) => RepoUrl::new(url),
        None => RepoUrl::from_repo(&repo),
    }?;
    match backend {
        GitBackend::Github(github) => {
            let git_client = GitClient::new(crate::GitBackend::Github(github))?;
            git_client.create_release(&git_tag, release_body).await?;
        }
        GitBackend::Gitea(gitea) => {
            let git_client = GitClient::new(GitBackend::Gitea(gitea))?;
            git_client.create_release(&git_tag, release_body).await?;
        }
    }
    Ok(())
}
