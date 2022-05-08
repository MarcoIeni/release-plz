use walkdir::WalkDir;

use crate::CARGO_TOML;
use std::{
    collections::hash_map::DefaultHasher,
    ffi::OsStr,
    fs::{self, File},
    hash::{Hash, Hasher},
    io::{self, Read},
    path::Path,
};

pub fn are_packages_equal(local_package: &Path, registry_package: &Path) -> anyhow::Result<bool> {
    if !are_cargo_toml_equal(local_package, registry_package) {
        return Ok(false);
    }

    let walker = WalkDir::new(local_package)
        .into_iter()
        .filter_entry(|e| {
            !((e.file_type().is_dir() && e.path().file_name() == Some(OsStr::new(".git")))
                || e.path_is_symlink())
        })
        .filter_map(Result::ok)
        .filter(|e| !(e.file_type().is_dir() && e.path() == local_package))
        .filter(|e| !{
            !e.file_type().is_dir()
                && (e.path().file_name() == Some(OsStr::new(".cargo_vcs_info.json"))
                    || e.path().file_name() == Some(OsStr::new(CARGO_TOML)))
        });

    for entry in walker {
        let path_without_prefix = entry.path().strip_prefix(local_package)?;
        let file_in_second_path = registry_package.join(path_without_prefix);
        if entry.file_type().is_dir() {
            let dir1 = fs::read_dir(entry.path())?;
            let dir2 = fs::read_dir(entry.path())?;
            if dir1.count() != dir2.count() {
                return Ok(false);
            }
        } else if !entry.file_type().is_dir() {
            if !file_in_second_path.is_file() {
                return Ok(false);
            }
            if !are_files_equal(entry.path(), &file_in_second_path)? {
                return Ok(false);
            }
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

fn are_files_equal(first: &Path, second: &Path) -> io::Result<bool> {
    let hash1 = file_hash(first)?;
    let hash2 = file_hash(second)?;
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
