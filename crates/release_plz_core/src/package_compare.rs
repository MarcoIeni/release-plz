use crate::CARGO_TOML;
use std::{
    collections::hash_map::DefaultHasher,
    ffi::OsStr,
    fs::{self, File},
    hash::{Hash, Hasher},
    io::{self, Read},
    path::{Path, PathBuf},
};

fn is_dir(entry: &ignore::DirEntry) -> bool {
    match entry.file_type() {
        Some(ft) => ft.is_dir(),
        None => false,
    }
}

/// Check if two packages are equal.
///
/// ## Args
/// - `ignored_dirs`: Directories of the `local_package` to ignore when comparing packages.
pub fn are_packages_equal(
    local_package: &Path,
    registry_package: &Path,
    ignored_dirs: Vec<PathBuf>,
) -> anyhow::Result<bool> {
    if !are_cargo_toml_equal(local_package, registry_package) {
        return Ok(false);
    }

    // Recursively traverse directories ignoring files present in `.gitignore`.
    // We ignore ignored files because we don't want to compare local files that are
    // not present in the package (such as `.DS_Store` or `Cargo.lock`, that might be generated
    // for libraries)
    let walker = ignore::WalkBuilder::new(local_package)
        // Read hidden files
        .hidden(false)
        // Don't consider `.ignore` files.
        .ignore(false)
        .filter_entry(move |e| {
            let ignored_dirs: Vec<&Path> = ignored_dirs.iter().map(|p| p.as_path()).collect();
            !((is_dir(e)
                && (e.path().file_name() == Some(OsStr::new(".git"))
                    || ignored_dirs.contains(&e.path())))
                || e.path_is_symlink())
        })
        .build()
        .filter_map(Result::ok)
        .filter(|e| !(is_dir(e) && e.path() == local_package))
        .filter(|e| !{
            !is_dir(e)
                && (e.path().file_name() == Some(OsStr::new(".cargo_vcs_info.json"))
                    || e.path().file_name() == Some(OsStr::new(CARGO_TOML)))
        });

    for entry in walker {
        let path_without_prefix = entry.path().strip_prefix(local_package)?;
        let file_in_second_path = registry_package.join(path_without_prefix);
        if is_dir(&entry) {
            let dir1 = fs::read_dir(entry.path())?;
            let dir2 = fs::read_dir(entry.path())?;
            if dir1.count() != dir2.count() {
                return Ok(false);
            }
        } else {
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
