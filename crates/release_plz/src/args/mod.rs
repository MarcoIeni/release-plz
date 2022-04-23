mod release_pr;
mod update;

use self::{release_pr::ReleasePr, update::Update};

#[derive(clap::Parser, Debug)]
#[clap(about, version, author)]
pub struct CliArgs {
    #[clap(subcommand)]
    pub command: Command,
}

#[derive(clap::Subcommand, Debug)]
pub enum Command {
    /// Update packages version and changelogs based on commit messages.
    Update(Update),
    /// Create a Pull Request representing the next release.
    /// The Pull request contains updated packages version and changelog based on commit messages.
    /// Close old PRs opened by release-plz, too.
    ReleasePr(ReleasePr),
}
