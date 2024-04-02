use std::collections::BTreeMap;

use anyhow::Context;
use cargo_metadata::{camino::Utf8Path, Package};
use git_cmd::git_in_dir;
use tempfile::{tempdir, TempDir};

use crate::{cargo_vcs_info, download, next_ver, PackagePath};

pub struct PackagesCollection {
    packages: BTreeMap<String, RegistryPackage>,
    /// Packages might be downloaded and stored in a temporary directory.
    /// The directory is stored here so that it is deleted on drop
    _temp_dir: Option<TempDir>,
}

pub struct RegistryPackage {
    pub package: Package,
    /// The SHA1 hash of the commit when the package was published.
    sha1: Option<String>,
}

impl RegistryPackage {
    pub fn published_at_sha1(&self) -> Option<&str> {
        self.sha1.as_deref()
    }
}

impl PackagesCollection {
    pub fn get_package(&self, package_name: &str) -> Option<&Package> {
        self.packages.get(package_name).map(|p| &p.package)
    }

    pub fn get_registry_package(&self, package_name: &str) -> Option<&RegistryPackage> {
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
            next_ver::publishable_packages_from_manifest(manifest)?
                .into_iter()
                .map(|p| RegistryPackage {
                    package: p,
                    sha1: None,
                })
                .collect(),
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
            let registry_packages = initialize_registry_package(registry_packages)?;
            (Some(temp_dir), registry_packages)
        }
    };
    let registry_packages: BTreeMap<String, RegistryPackage> = registry_packages
        .into_iter()
        .map(|c| {
            let package_name = c.package.name.clone();
            (package_name, c)
        })
        .collect();
    Ok(PackagesCollection {
        _temp_dir: temp_dir,
        packages: registry_packages,
    })
}

fn initialize_registry_package(packages: Vec<Package>) -> anyhow::Result<Vec<RegistryPackage>> {
    let mut registry_packages = vec![];
    for p in packages {
        let package_path = p.package_path().unwrap();
        let cargo_vcs_info_path = package_path.join(".cargo_vcs_info.json");
        // cargo_vcs_info is only present if `cargo publish` wasn't used with
        // the `--allow-dirty` flag inside a git repo.
        let sha1 = if cargo_vcs_info_path.exists() {
            let sha1 = cargo_vcs_info::read_sha1_from_cargo_vcs_info(&cargo_vcs_info_path);
            // Remove the file, otherwise `cargo publish --list` fails
            fs_err::remove_file(cargo_vcs_info_path)?;
            sha1
        } else {
            None
        };
        let git_repo = package_path.join(".git");
        if !git_repo.exists() {
            git_in_dir(package_path, &["init"])?;
            git_in_dir(package_path, &["add", "."])?;
            git_in_dir(package_path, &["commit", "-m", "init"]).unwrap();
        }
        registry_packages.push(RegistryPackage { package: p, sha1 });
    }
    Ok(registry_packages)
}
