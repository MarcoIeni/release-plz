use anyhow::Context;
use tracing::debug;

use crate::{cargo::run_cargo, CARGO_TOML};
use std::{
    collections::hash_map::DefaultHasher,
    ffi::OsStr,
    fs::File,
    hash::{Hash, Hasher},
    io::{self, Read},
    path::{Path, PathBuf},
};

/// Check if two packages are equal.
///
/// ## Args
/// - `ignored_dirs`: Directories of the `local_package` to ignore when comparing packages.
pub fn are_packages_equal(
    local_package: &Path,
    registry_package: &Path,
    ignored_dirs: Vec<PathBuf>,
) -> anyhow::Result<bool> {
    debug!(
        "compare local package {:?} with registry package {:?}",
        local_package, registry_package
    );
    if !are_cargo_toml_equal(local_package, registry_package) {
        return Ok(false);
    }

    // When a package is published to a cargo registry, the original `Cargo.toml` file is stored as `Cargo.toml.orig`.
    // We need to rename it to `Cargo.toml.orig.orig`, because this name is reserved, and `cargo package` will fail if it exists.
    rename(
        registry_package.join("Cargo.toml.orig"),
        registry_package.join("Cargo.toml.orig.orig"),
    )?;

    let local_package_stdout = run_cargo_package(local_package).with_context(|| {
        format!("cannot determine packaged files of local package {local_package:?}")
    })?;
    let registry_package_stdout = run_cargo_package(registry_package).with_context(|| {
        format!("cannot determine packaged files of registry package {registry_package:?}")
    })?;

    // Rename the file to the original name.
    rename(
        registry_package.join("Cargo.toml.orig.orig"),
        registry_package.join("Cargo.toml.orig"),
    )?;

    let local_files = local_package_stdout
        .lines()
        .filter(|file| *file != ".cargo_vcs_info.json");

    let registry_files = registry_package_stdout
        .lines()
        .filter(|file| *file != "Cargo.toml.orig.orig" || *file != ".cargo_vcs_info.json");

    if !local_files.clone().eq(registry_files) {
        // New files were added or removed.
        debug!("cargo package list is different");
        return Ok(false);
    }

    let ignored_dirs: Vec<&Path> = ignored_dirs.iter().map(|p| p.as_path()).collect();
    let files = local_files
        .map(|file| local_package.join(file))
        .filter(|file| {
            let should_ignore_file = ignored_dirs
                .iter()
                .any(|directory| file.starts_with(directory));
            !(should_ignore_file
            || file.is_symlink()
            // Ignore `Cargo.lock` because the local one is different from the published one in workspaces.
            || file.file_name() == Some(OsStr::new("Cargo.lock"))
            // Ignore `Cargo.toml` because we already checked it before.
            || file.file_name() == Some(OsStr::new(CARGO_TOML))
            // Ignore `Cargo.toml.orig` because it's auto generated.
            || file.file_name() == Some(OsStr::new("Cargo.toml.orig")))
        });

    for file in files {
        let file_in_second_path = registry_package.join(&file);

        if !are_files_equal(&file, &file_in_second_path).context("files are not equal")? {
            return Ok(false);
        }
    }

    Ok(true)
}

fn rename(from: impl AsRef<Path>, to: impl AsRef<Path>) -> anyhow::Result<()> {
    let from = from.as_ref();
    let to = to.as_ref();
    std::fs::rename(from, to).with_context(|| format!("cannot rename {from:?} to {to:?}"))
}

fn run_cargo_package(package: &Path) -> anyhow::Result<String> {
    let (stdout, stderr) = run_cargo(package, &["package", "--list", "--quiet"])
        .context("cannot run `cargo package`")?;

    if !stderr.is_empty() {
        anyhow::bail!("error while running `cargo package`: {stderr}");
    }

    Ok(stdout)
}

fn are_cargo_toml_equal(local_package: &Path, registry_package: &Path) -> bool {
    // When a package is published to a cargo registry, the original `Cargo.toml` file is stored as
    // `Cargo.toml.orig`
    let cargo_orig = format!("{CARGO_TOML}.orig");
    are_files_equal(
        &local_package.join(CARGO_TOML),
        &registry_package.join(cargo_orig),
    )
    .unwrap_or(false)
}

fn are_files_equal(first: &Path, second: &Path) -> anyhow::Result<bool> {
    let hash1 = file_hash(first).with_context(|| format!("cannot determine hash of {first:?}"))?;
    let hash2 =
        file_hash(second).with_context(|| format!("cannot determine hash of {second:?}"))?;
    Ok(hash1 == hash2)
}

fn file_hash(file: &Path) -> io::Result<u64> {
    let buffer = &mut vec![];
    File::open(file)?.read_to_end(buffer)?;
    let mut hasher = DefaultHasher::new();
    buffer.hash(&mut hasher);
    let hash = hasher.finish();
    Ok(hash)
}
