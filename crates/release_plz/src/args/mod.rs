mod generate_completions;
mod release;
mod release_pr;
mod repo_command;
mod update;

use std::path::{Path, PathBuf};

use anyhow::Context;
use release_plz_core::CARGO_TOML;
use tracing::info;

use crate::config::Config;

use self::{
    generate_completions::GenerateCompletions, release::Release, release_pr::ReleasePr,
    update::Update,
};

#[derive(clap::Parser, Debug)]
#[command(about, version, author)]
pub struct CliArgs {
    #[command(subcommand)]
    pub command: Command,
    /// Print source location and additional information in logs.
    /// To change the log level, use the `RUST_FLAG` environment variable.
    #[arg(short, long, global = true)]
    pub verbose: bool,
}

#[derive(clap::Subcommand, Debug)]
pub enum Command {
    /// Update packages version and changelogs based on commit messages.
    Update(Update),
    /// Create a Pull Request representing the next release.
    /// The Pull request contains updated packages version and changelog based on commit messages.
    /// Close old PRs opened by release-plz, too.
    ReleasePr(ReleasePr),
    /// For each package not published to the cargo registry yet:
    /// - create and push upstream a tag in the format of `<package>-v<version>`.
    /// - publish the package to the cargo registry.
    /// You can run this command in the CI on every commit in the main branch.
    Release(Release),
    /// Generate command autocompletions for various shells.
    GenerateCompletions(GenerateCompletions),
    /// Check if a newer version of release-plz is available.
    CheckUpdates,
}

fn local_manifest(project_manifest: Option<&Path>) -> PathBuf {
    match project_manifest {
        Some(manifest) => manifest.to_path_buf(),
        None => std::env::current_dir()
            .expect("cannot retrieve current directory")
            .join(CARGO_TOML),
    }
}

fn parse_config(
    config_path: Option<&Path>,
    project_manifest: Option<&Path>,
) -> anyhow::Result<Config> {
    let (config, path) = if let Some(config_path) = config_path {
        match std::fs::read_to_string(config_path) {
            Ok(config) => (config, config_path),
            Err(e) => match e.kind() {
                std::io::ErrorKind::NotFound => {
                    anyhow::bail!("specified config does not exist at path {config_path:?}")
                }
                _ => anyhow::bail!("can't read {config_path:?}: {e:?}"),
            },
        }
    } else {
        match first_file_contents([
            Path::new("release-plz.toml"),
            Path::new(".release-plz.toml"),
        ])? {
            Some((config, path)) => (config, path),
            None => {
                if let Some(project_manifest) = project_manifest.filter(|v| v.exists()) {
                    return match parse_config_from_metadata(project_manifest)? {
                        Some(config) => {
                            info!(
                                "using configuration from metadata in {:?}",
                                project_manifest
                            );
                            Ok(config)
                        }
                        None => Ok(Config::default()),
                    };
                } else {
                    info!("release-plz config file not found, using default configuration");
                    return Ok(Config::default());
                }
            }
        }
    };

    info!("using release-plz config file {}", path.display());
    toml::from_str(&config).with_context(|| format!("invalid config file {config_path:?}"))
}

fn parse_config_from_metadata(project_manifest: &Path) -> anyhow::Result<Option<Config>> {
    let metadata = cargo_metadata::MetadataCommand::new()
        .no_deps()
        .manifest_path(project_manifest)
        .exec()
        .context("failed to execute cargo_metadata")?;
    // check both workspace and package metadata
    for metadata in [
        Some(metadata.clone().workspace_metadata),
        metadata.packages.first().map(|v| v.metadata.clone()),
    ]
    .into_iter()
    .filter(|v| v.clone().is_some_and(|v| !v.is_null()))
    {
        if let Some(metadata) = metadata
            // safe to unwrap() since it is checked above
            .unwrap()
            .as_object()
            .ok_or_else(|| anyhow::anyhow!("failed to convert metadata to object"))?
            .get("release-plz")
        {
            let config: Config = serde_json::from_value(metadata.clone())
                .context("failed to parse config from metadata")?;
            return Ok(Some(config));
        }
    }
    Ok(None)
}

/// Returns the contents of the first file that exists.
///
/// If none of the files exist, returns `Ok(None)`.
///
/// # Errors
///
/// Errors if opening and reading one of files paths fails for reasons other that it doesn't exist.
fn first_file_contents<'a>(
    paths: impl IntoIterator<Item = &'a Path>,
) -> anyhow::Result<Option<(String, &'a Path)>> {
    let paths = paths.into_iter();

    for path in paths {
        match std::fs::read_to_string(path) {
            Ok(config) => return Ok(Some((config, path))),
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {}
            Err(err) => return Err(err.into()),
        }
    }

    Ok(None)
}
