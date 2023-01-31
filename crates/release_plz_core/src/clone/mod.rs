// Copied from [cargo-clone](https://github.com/JanLikar/cargo-clone/blob/89ba4da215663ffb3b8c93a674f3002937eafec4/cargo-clone-core/src/lib.rs)
//! Fetch the source code of a Rust crate from a registry.

#![warn(missing_docs)]

mod cloner_builder;
mod source;

pub use cloner_builder::*;
pub use source::*;
use tracing::warn;

use std::collections::HashSet;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

use anyhow::{bail, Context};

use cargo::core::dependency::Dependency;
use cargo::core::source::Source;
use cargo::core::Package;
use cargo::core::QueryKind;
use cargo::sources::{PathSource, SourceConfigMap};

use walkdir::WalkDir;

// Re-export cargo types.
pub use cargo::{
    core::SourceId,
    util::{CargoResult, Config},
};

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
    pub(crate) config: Config,
    /// Directory where the crates will be cloned.
    /// Each crate is cloned into a subdirectory of this directory.
    pub(crate) directory: PathBuf,
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
    pub fn clone(&self, crates: &[Crate]) -> CargoResult<Vec<(Package, PathBuf)>> {
        let _lock = self.config.acquire_package_cache_lock()?;

        let mut src = get_source(self.srcid, &self.config)?;
        let mut cloned_pkgs = vec![];

        for crate_ in crates {
            let mut dest_path = self.directory.clone();

            dest_path.push(&crate_.name);

            self.clone_in(crate_, &dest_path, &mut src)?.map(|pkg| {
                cloned_pkgs.push((pkg, dest_path));
            });
        }

        Ok(cloned_pkgs)
    }

    fn clone_in<'a, T>(
        &self,
        crate_: &Crate,
        dest_path: &Path,
        src: &mut T,
    ) -> CargoResult<Option<Package>>
    where
        T: Source + 'a,
    {
        if !dest_path.exists() {
            fs::create_dir_all(dest_path)?;
        }

        self.config
            .shell()
            .verbose(|s| s.note(format!("Cloning into {:?}", &self.directory)))?;

        // Cloning into an existing directory is only allowed if the directory is empty.
        let is_empty = dest_path.read_dir()?.next().is_none();
        if !is_empty {
            bail!(
                "destination path '{}' already exists and is not an empty directory.",
                dest_path.display()
            );
        }

        self.clone_single(crate_, dest_path, src)
    }

    fn clone_single<'a, T>(
        &self,
        crate_: &Crate,
        dest_path: &Path,
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
                    clone_directory(pkg.root(), dest_path)?;
                }
                Some(pkg)
            }
            None => {
                warn!("Package `{}` not found", crate_.name);
                None
            }
        };
        Ok(pkg)
    }
}

fn get_source<'a>(srcid: SourceId, config: &'a Config) -> CargoResult<Box<dyn Source + 'a>> {
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
    config: &Config,
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
        match src.query(&dep, QueryKind::Exact, &mut |summary| {
            summaries.push(summary)
        })? {
            std::task::Poll::Ready(()) => break,
            std::task::Poll::Pending => src.block_until_ready()?,
        }
    }

    let latest = summaries.iter().max_by_key(|s| s.version());

    let pkg = match latest {
        Some(l) => {
            config
                .shell()
                .note(format!("Downloading {} {}", name, l.version()))?;
            let pkg = Box::new(src).download_now(l.package_id(), config)?;
            Some(pkg)
        }
        None => None,
    };
    Ok(pkg)
}

// clone_directory copies the contents of one directory into another directory, which must
// already exist.
fn clone_directory(from: &Path, to: &Path) -> CargoResult<()> {
    if !to.is_dir() {
        bail!("Not a directory: {}", to.to_string_lossy());
    }
    for entry in WalkDir::new(from) {
        let entry = entry.unwrap();
        let file_type = entry.file_type();
        let mut dest_path = to.to_owned();
        dest_path.push(entry.path().strip_prefix(from).unwrap());

        if entry.file_name() == ".cargo-ok" {
            continue;
        }

        if !file_type.is_dir() {
            // .cargo-ok is not wanted in this context
            fs::copy(entry.path(), &dest_path)?;
        } else if file_type.is_dir() {
            if dest_path == to {
                continue;
            }
            fs::create_dir(&dest_path)?;
        }
    }

    Ok(())
}

fn clone_git_repo(repo: &str, to: &Path) -> CargoResult<()> {
    let status = Command::new("git")
        .arg("clone")
        .arg(repo)
        .arg(to.to_str().unwrap())
        .status()
        .context("Failed to clone from git repo.")?;

    if !status.success() {
        bail!("Failed to clone from git repo.")
    }

    Ok(())
}
