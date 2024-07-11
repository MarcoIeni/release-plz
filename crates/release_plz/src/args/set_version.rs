use std::path::{Path, PathBuf};

use cargo_metadata::semver::Version;
use clap::builder::PathBufValueParser;
use release_plz_core::{SetVersionRequest, VersionChange};

use super::manifest_command::ManifestCommand;

#[derive(clap::Parser, Debug)]
pub struct SetVersion {
    /// New version of the package you want to update. Format: `<package_name>@<version-req>`.
    pub versions: Vec<String>,
    /// Path to the Cargo.toml of the project you want to update.
    /// If not provided, release-plz will use the Cargo.toml of the current directory.
    /// Both Cargo workspaces and single packages are supported.
    #[arg(long, value_parser = PathBufValueParser::new())]
    manifest_path: Option<PathBuf>,
}

impl SetVersion {
    fn parse_versions(self) -> anyhow::Result<Vec<VersionChange>> {
        self
            .versions
            .iter()
            .map(|v| {
                let error = Err(anyhow::anyhow!("version {v} is invalid. Format need to be `<package_name>@<version>`. E.g. `release-plz set-version serde@1.2.3`"));
                let d: Vec<_> = v.split('@').collect();
                #[allow(clippy::get_first)]
                let Some(package) = d.get(0) else {return error;};
                let Some(version) = d.get(1) else {return error;};
                let version = Version::parse(version).unwrap();
                Ok(VersionChange::new(package.to_string(), version ))
            })
            .collect()
    }

    /// Get [`SetVersionRequest`]
    pub fn set_version_request(self) -> anyhow::Result<SetVersionRequest> {
        let cargo_metadata = self.cargo_metadata()?;
        let version_changes = self.parse_versions()?;
        Ok(SetVersionRequest::new(version_changes, metadata))
    }
}

impl ManifestCommand for SetVersion {
    fn optional_manifest(&self) -> Option<&Path> {
        self.manifest_path.as_deref()
    }
}
