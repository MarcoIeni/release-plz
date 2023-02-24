mod generate_completions;
mod release;
mod release_pr;
mod update;

use std::path::{Path, PathBuf};

use release_plz_core::CARGO_TOML;

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
    /// - create and push upstream a tag in the format of <package>-v<version>.
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
