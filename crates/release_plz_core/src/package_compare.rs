use anyhow::Context;
use tracing::debug;

use crate::{cargo::run_cargo, CARGO_TOML};
use std::{
    collections::hash_map::DefaultHasher,
    ffi::OsStr,
    fs::{rename, File},
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

    rename(
        &registry_package.join("Cargo.toml.orig"),
        &registry_package.join("Cargo.toml.orig.orig"),
    )?;

    let (local_stdout, local_stderr) = run_cargo(local_package, &["package", "--list", "-q"])
        .context("cannot run cargo package on local package")?;
    let (registry_stdout, registry_stderr) =
        run_cargo(registry_package, &["package", "--list", "-q"])
            .context("cannot run cargo package on registry package")?;

    rename(
        &registry_package.join("Cargo.toml.orig.orig"),
        &registry_package.join("Cargo.toml.orig"),
    )
    .context("cannot rename Cargo.toml.orig.orig")?;

    if !local_stderr.is_empty() {
        anyhow::bail!("stderr of cargo package not empty - local: {local_stderr}");
    }

    if !registry_stderr.is_empty() {
        anyhow::bail!("stderr of cargo package not empty - registry: {registry_stderr}");
    }

    let local_files = local_stdout
        .lines()
        .filter(|file| file.to_string() != ".cargo_vcs_info.json");

    let registry_files = registry_stdout
        .lines()
        .filter(|file| file.to_string() != "Cargo.toml.orig.orig");

    if !local_files.clone().eq(registry_files) {
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
            || file.file_name() == Some(OsStr::new("Cargo.toml.orig")))
        });

    for file in files {
        let file_path = file.as_path();

        let file_in_second_path = registry_package.join(file_path);

        if !are_files_equal(file_path, &file_in_second_path).context("files are not equal")? {
            return Ok(false);
        }
    }

    Ok(true)
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
