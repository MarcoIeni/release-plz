use std::collections::BTreeMap;

use anyhow::Context;
use cargo_metadata::{camino::Utf8Path, Package};
use git_cmd::git_in_dir;
use tempfile::{tempdir, TempDir};

use crate::{download, next_ver, PackagePath};

pub struct PackagesCollection {
    packages: BTreeMap<String, Package>,
    /// Packages might be downloaded and stored in a temporary directory.
    /// The directory is stored here so that it is deleted on drop
    _temp_dir: Option<TempDir>,
}

impl PackagesCollection {
    pub fn get_package(&self, package_name: &str) -> Option<&Package> {
        self.packages.get(package_name)
    }
}

pub fn get_registry_packages(
    registry_manifest: Option<&Utf8Path>,
    local_packages: &[&Package],
    registry: Option<&str>,
) -> anyhow::Result<PackagesCollection> {
    let (temp_dir, registry_packages) = match registry_manifest {
        Some(manifest) => (
            None,
            next_ver::publishable_packages_from_manifest(manifest)?,
        ),
        None => {
            let temp_dir = tempdir().context("failed to get a temporary directory")?;
            let local_packages_names: Vec<&str> =
                local_packages.iter().map(|c| c.name.as_str()).collect();
            let directory = temp_dir.as_ref().to_str().context("invalid tempdir path")?;

            let mut downloader = download::PackageDownloader::new(local_packages_names, directory);
            if let Some(registry) = registry {
                downloader = downloader.with_registry(registry.to_string());
            }
            let registry_packages = downloader
                .download()
                .context("failed to download packages")?;

            // After downloading the package, we initialize a git repo in the package.
            // This is because if cargo doesn't find a git repo in the package, it doesn't
            // show hidden files in `cargo package --list` output.
            initialize_git_repo(&registry_packages)?;
            (Some(temp_dir), registry_packages)
        }
    };
    let registry_packages: BTreeMap<String, Package> = registry_packages
        .into_iter()
        .map(|c| {
            let package_name = c.name.clone();
            (package_name, c)
        })
        .collect();
    Ok(PackagesCollection {
        _temp_dir: temp_dir,
        packages: registry_packages,
    })
}

fn initialize_git_repo<'a>(packages: &[Package]) -> anyhow::Result<()> {
    for p in packages {
        let package_path = p.package_path().unwrap();
        let git_repo = package_path.join(".git");
        if !git_repo.exists() {
            git_in_dir(package_path, &["init"])?;
            git_in_dir(package_path, &["add", "."])?;
            git_in_dir(package_path, &["commit", "-m", "init"]).unwrap();
        }
    }
    Ok(())
}
