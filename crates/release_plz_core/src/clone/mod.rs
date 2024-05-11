// Copied from [cargo-clone](https://github.com/JanLikar/cargo-clone/blob/89ba4da215663ffb3b8c93a674f3002937eafec4/cargo-clone-core/src/lib.rs)
//! Fetch the source code of a Rust crate from a registry.

#![warn(missing_docs)]

mod cloner_builder;
mod source;

use cargo::util::cache_lock::CacheLockMode;
use cargo_metadata::camino::Utf8Path;
use cargo_metadata::camino::Utf8PathBuf;
pub use cloner_builder::*;
pub use source::*;
use tracing::warn;

use std::collections::HashSet;

use std::process::Command;

use anyhow::{bail, Context};

use cargo::core::dependency::Dependency;
use cargo::core::Package;
use cargo::sources::source::{QueryKind, Source};
use cargo::sources::{PathSource, SourceConfigMap};

use walkdir::WalkDir;

// Re-export cargo types.
pub use cargo::{
    core::SourceId,
    util::{CargoResult, GlobalContext},
};

use crate::fs_utils::strip_prefix;
use crate::fs_utils::to_utf8_path;

/// Rust crate.
#[derive(PartialEq, Eq, Debug)]
pub struct Crate {
    name: String,
    version: Option<String>,
}

impl Crate {
    /// Create a new [`Crate`].
    /// If `version` is not specified, the latest version is chosen.
    pub fn new(name: String, version: Option<String>) -> Crate {
        Crate { name, version }
    }
}

/// Clones a crate.
pub struct Cloner {
    /// Cargo configuration.
    pub(crate) config: GlobalContext,
    /// Directory where the crates will be cloned.
    /// Each crate is cloned into a subdirectory of this directory.
    pub(crate) directory: Utf8PathBuf,
    /// Where the crates will be cloned from.
    pub(crate) srcid: SourceId,
    /// If true, use `git` to clone the git repository present in the manifest metadata.
    pub(crate) use_git: bool,
}

impl Cloner {
    /// Creates a new [`ClonerBuilder`] that:
    /// - Uses crates.io as source.
    /// - Clones the crates into the current directory.
    pub fn builder() -> ClonerBuilder {
        ClonerBuilder::new()
    }

    /// Clone the specified crates from registry or git repository.
    /// Each crate is cloned in a subdirectory named as the crate name.
    /// Returns the cloned crates and the path where they are cloned.
    /// If a crate doesn't exist, is not returned.
    pub fn clone(&self, crates: &[Crate]) -> CargoResult<Vec<(Package, Utf8PathBuf)>> {
        let _lock = self
            .config
            .acquire_package_cache_lock(CacheLockMode::DownloadExclusive)?;

        let mut src = get_source(self.srcid, &self.config)?;
        let mut cloned_pkgs = vec![];

        for crate_ in crates {
            let mut dest_path = self.directory.clone();

            dest_path.push(&crate_.name);

            if let Some(pkg) = self
                .clone_in(crate_, &dest_path, &mut src)
                .with_context(|| {
                    format!("failed to clone package {} in {dest_path}", &crate_.name)
                })?
            {
                cloned_pkgs.push((pkg, dest_path));
            }
        }

        Ok(cloned_pkgs)
    }

    fn clone_in<'a, T>(
        &self,
        crate_: &Crate,
        dest_path: &Utf8Path,
        src: &mut T,
    ) -> CargoResult<Option<Package>>
    where
        T: Source + 'a,
    {
        if !dest_path.exists() {
            fs_err::create_dir_all(dest_path)?;
        }

        self.config
            .shell()
            .verbose(|s| s.note(format!("Cloning into {:?}", &self.directory)))?;

        // Cloning into an existing directory is only allowed if the directory is empty.
        let is_empty = dest_path.read_dir()?.next().is_none();
        if !is_empty {
            bail!(
                "destination path '{}' already exists and is not an empty directory.",
                dest_path
            );
        }

        self.clone_single(crate_, dest_path, src)
    }

    fn clone_single<'a, T>(
        &self,
        crate_: &Crate,
        dest_path: &Utf8Path,
        src: &mut T,
    ) -> CargoResult<Option<Package>>
    where
        T: Source + 'a,
    {
        let pkg = match select_pkg(&self.config, src, &crate_.name, crate_.version.as_deref())? {
            Some(pkg) => {
                if self.use_git {
                    let repo = &pkg.manifest().metadata().repository;

                    if repo.is_none() {
                        bail!(
                    "Cannot clone {} from git repo because it is not specified in package's manifest.",
                    &crate_.name
                )
                    }

                    clone_git_repo(repo.as_ref().unwrap(), dest_path)?;
                } else {
                    clone_directory(to_utf8_path(pkg.root())?, dest_path)
                        .context("failed to clone directory")?;
                }
                Some(pkg)
            }
            None => None,
        };
        Ok(pkg)
    }
}

fn get_source<'a>(srcid: SourceId, config: &'a GlobalContext) -> CargoResult<Box<dyn Source + 'a>> {
    let mut source = if srcid.is_path() {
        let path = srcid.url().to_file_path().expect("path must be valid");
        Box::new(PathSource::new(&path, srcid, config))
    } else {
        let map = SourceConfigMap::new(config)?;
        map.load(srcid, &HashSet::default())?
    };

    source.invalidate_cache();
    Ok(source)
}

fn select_pkg<'a, T>(
    config: &GlobalContext,
    src: &mut T,
    name: &str,
    vers: Option<&str>,
) -> CargoResult<Option<Package>>
where
    T: Source + 'a,
{
    let dep = Dependency::parse(name, vers, src.source_id())?;
    let mut summaries = vec![];
    loop {
        let query_result = src.query(&dep, QueryKind::Exact, &mut |summary| {
            summaries.push(summary);
        });
        match query_result {
            std::task::Poll::Ready(res) => match res {
                Ok(()) => break,
                Err(err) => {
                    return package_from_query_err(err);
                }
            },
            std::task::Poll::Pending => match src.block_until_ready() {
                Ok(()) => {}
                Err(err) => {
                    return package_from_query_err(err);
                }
            },
        }
    }

    let latest = summaries.iter().max_by_key(|s| s.as_summary().version());

    let pkg = match latest {
        Some(l) => {
            config
                .shell()
                .note(format!("Downloading {} {}", name, l.as_summary().version()))?;
            let pkg = Box::new(src).download_now(l.package_id(), config)?;
            Some(pkg)
        }
        None => {
            warn!("Package `{}@{}` not found", name, vers.unwrap_or("*.*.*"));
            None
        }
    };
    Ok(pkg)
}

fn package_from_query_err(err: anyhow::Error) -> CargoResult<Option<Package>> {
    if err.to_string().contains("failed to fetch") {
        // I observed this error happens when the cargo registry contains no crates.
        // If this isn't the case, open an issue.
        warn!("Failed to fetch package from registry. I assume the registry is empty.");
        Ok(None)
    } else {
        Err(err)
    }
}

// clone_directory copies the contents of one directory into another directory, which must
// already exist.
fn clone_directory(from: &Utf8Path, to: &Utf8Path) -> CargoResult<()> {
    if !to.is_dir() {
        bail!("Not a directory: {to}");
    }
    for entry in WalkDir::new(from) {
        let entry = entry.unwrap();
        let file_type = entry.file_type();
        let mut dest_path = to.to_owned();
        let utf8_entry: &Utf8Path = entry.path().try_into()?;
        dest_path.push(strip_prefix(utf8_entry, from).unwrap());

        if entry.file_name() == ".cargo-ok" {
            continue;
        }

        if !file_type.is_dir() {
            // .cargo-ok is not wanted in this context
            fs_err::copy(entry.path(), &dest_path)?;
        } else if file_type.is_dir() {
            if dest_path == to {
                continue;
            }
            fs_err::create_dir(&dest_path)?;
        }
    }

    Ok(())
}

fn clone_git_repo(repo: &str, to: &Utf8Path) -> CargoResult<()> {
    let status = Command::new("git")
        .arg("clone")
        .arg(repo)
        .arg(to)
        .status()
        .context("Failed to clone from git repo.")?;

    if !status.success() {
        bail!("Failed to clone from git repo.")
    }

    Ok(())
}
