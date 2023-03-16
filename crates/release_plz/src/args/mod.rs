mod generate_completions;
mod release;
mod release_pr;
mod update;

use std::path::{Path, PathBuf};

use anyhow::Context;
use clap::builder::PathBufValueParser;
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
    #[arg(short, long)]
    pub verbose: bool,
    /// Path to the release-plz yaml config file.
    /// Default: `./release-plz.yaml`.
    /// If no config file is found, the default configuration is used.
    #[arg(
        long,
        value_name = "PATH",
        value_parser = PathBufValueParser::new()
    )]
    config: Option<PathBuf>,
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

impl CliArgs {
    pub fn config(&self) -> anyhow::Result<Config> {
        let config_path = self.config.clone().unwrap_or("release-plz.yaml".into());

        match std::fs::read_to_string(&config_path) {
            Ok(config) => {
                info!("using release-plz config file {}", config_path.display());
                toml::from_str(&config)
                    .with_context(|| format!("invalid config file {config_path:?}"))
            }
            Err(e) => match e.kind() {
                std::io::ErrorKind::NotFound => {
                    info!("release-plz config file not found, using default configuration");
                    Ok(Config::default())
                }
                _ => anyhow::bail!("can't read {config_path:?}: {e:?}"),
            },
        }
    }
}
