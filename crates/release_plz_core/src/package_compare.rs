use anyhow::Context;
use cargo_metadata::{
    camino::{Utf8Path, Utf8PathBuf},
    Package,
};
use tracing::debug;

use crate::{cargo::run_cargo, CARGO_TOML};
use std::{
    collections::hash_map::DefaultHasher,
    fs::File,
    hash::{Hash, Hasher},
    io::{self, Read},
    path::{Path},
};

/// Check if two packages are equal.
///
/// ## Args
/// - `ignored_dirs`: Directories of the `local_package` to ignore when comparing packages.
pub fn are_packages_equal(
    local_package: &Utf8Path,
    registry_package: &Utf8Path,
) -> anyhow::Result<bool> {
    debug!(
        "compare local package {:?} with registry package {:?}",
        local_package, registry_package
    );
    if !are_cargo_toml_equal(local_package, registry_package) {
        debug!("Cargo.toml is different");
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
        .filter(|file| *file != "Cargo.toml.orig" && *file != ".cargo_vcs_info.json");

    let registry_files = registry_package_stdout.lines().filter(|file| {
        *file != "Cargo.toml.orig"
            && *file != "Cargo.toml.orig.orig"
            && *file != ".cargo_vcs_info.json"
    });

    if !local_files.clone().eq(registry_files) {
        // New files were added or removed.
        debug!("cargo package list is different");
        return Ok(false);
    }

    let local_files = local_files
        .map(|file| local_package.join(file))
        .filter(|file| {
            !(file.is_symlink()
            // `cargo package --list` can return files that don't exist locally,
            // such as the `README.md` file if the `Cargo.toml` specified a different path.
            || !file.exists()
            // Ignore `Cargo.lock` because the local one is different from the published one in workspaces.
            || file.file_name() == Some("Cargo.lock")
            // Ignore `Cargo.toml` because we already checked it before.
            || file.file_name() == Some(CARGO_TOML)
            // Ignore `Cargo.toml.orig` because it's auto generated.
            || file.file_name() == Some("Cargo.toml.orig"))
        });

    for local_path in local_files {
        let relative_path = local_path
            .strip_prefix(local_package)
            .with_context(|| format!("can't find {local_package:?} prefix in {local_path:?}"))?;

        let registry_path = registry_package.join(relative_path);
        if !are_files_equal(&local_path, &registry_path).context("files are not equal")? {
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

fn run_cargo_package(package: &Utf8Path) -> anyhow::Result<String> {
    // we use `--allow-dirty` because we have `Cargo.toml.orig.orig`, which is an uncommitted change.
    let args = ["package", "--list", "--quiet", "--allow-dirty"];
    let output = run_cargo(package, &args).context("cannot run `cargo package`")?;

    anyhow::ensure!(
        output.status.success(),
        "error while running `cargo package`: {}",
        output.stderr
    );

    Ok(output.stdout)
}

fn are_cargo_toml_equal(local_package: &Utf8Path, registry_package: &Utf8Path) -> bool {
    // When a package is published to a cargo registry, the original `Cargo.toml` file is stored as
    // `Cargo.toml.orig`
    let cargo_orig = format!("{CARGO_TOML}.orig");
    are_files_equal(
        &local_package.join(CARGO_TOML),
        &registry_package.join(cargo_orig),
    )
    .unwrap_or(false)
}

/// Returns true if the README file of the local package is the same as the one in the registry.
/// Returns false if:
/// - the README is the same
/// - the local package doesn't have a `readme` field in the `Cargo.toml`.
/// - the package doesn't have a README at all.
pub fn is_readme_updated(
    local_package_path: &Utf8Path,
    package: &Package,
    registry_package_path: &Utf8Path,
) -> anyhow::Result<bool> {
    let local_package_readme_path = local_readme_override(package, local_package_path);
    let are_readmes_equal = match local_package_readme_path {
        Some(local_package_readme_path) => {
            let registry_package_readme_path = registry_package_path.join("README.md");
            if !registry_package_readme_path.exists() {
                return Ok(true);
            }
            are_files_equal(&local_package_readme_path, &registry_package_readme_path)
                .context("cannot compare README files")?
        }
        None => true,
    };
    Ok(!are_readmes_equal)
}

pub fn local_readme_override(
    package: &Package,
    local_package_path: &Utf8Path,
) -> Option<Utf8PathBuf> {
    package
        .readme
        .as_ref()
        .map(|readme| local_package_path.join(readme))
}

fn are_files_equal(first: &Utf8Path, second: &Utf8Path) -> anyhow::Result<bool> {
    let hash1 = file_hash(first).with_context(|| format!("cannot determine hash of {first:?}"))?;
    let hash2 =
        file_hash(second).with_context(|| format!("cannot determine hash of {second:?}"))?;
    Ok(hash1 == hash2)
}

fn file_hash(file: &Utf8Path) -> io::Result<u64> {
    let buffer = &mut vec![];
    File::open(file)?.read_to_end(buffer)?;
    let mut hasher = DefaultHasher::new();
    buffer.hash(&mut hasher);
    let hash = hasher.finish();
    Ok(hash)
}
