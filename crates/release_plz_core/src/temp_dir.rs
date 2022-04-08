use anyhow::{anyhow, Context};
use tempfile::TempDir;

/// Get a temporary directory depending on the environment you are in.
/// In Docker GitHub actions, you cannot access `/tmp`, so in this environment
/// this function creates a temporary directory inside `$HOME`.
pub fn temporary_directory() -> anyhow::Result<TempDir> {
    let dir = if std::env::var("GITHUB_ACTIONS").as_deref() == Ok("true") {
        let home = dirs::home_dir().ok_or_else(|| anyhow!("cannot retrieve home directory"))?;
        tempfile::tempdir_in(&home)
    } else {
        tempfile::tempdir()
    };
    let dir = dir.context("cannot create temporary directory")?;
    Ok(dir)
}
