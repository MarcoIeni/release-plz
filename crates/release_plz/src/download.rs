use std::path::Path;

use anyhow::Context;
use cargo::core::SourceId;
use cargo_metadata::Package;
use tempfile::tempdir;
use tracing::instrument;

#[instrument]
fn download_crate(crates: &[&str]) -> anyhow::Result<Vec<Package>> {
    let config = cargo::Config::default().expect("Unable to get cargo config.");
    let source_id = SourceId::crates_io(&config).expect("Unable to retriece source id.");
    let crates: Vec<cargo_clone::Crate> = crates
        .iter()
        .map(|c| cargo_clone::Crate::new(c.to_string(), None))
        .collect();
    let temp_dir = tempdir()?;
    let directory = Some(temp_dir.as_ref().to_str().expect("invalid path"));
    let clone_opts = cargo_clone::CloneOpts::new(&crates, &source_id, directory, false);
    cargo_clone::clone(&clone_opts, &config).context("cannot download remote crates")?;
    Ok(list_crates(temp_dir.as_ref()))
}

fn list_crates(directory: &Path) -> Vec<Package> {
    cargo_edit::workspace_members(Some(directory)).unwrap()
}
