use std::{path::Path, process::Command};

use anyhow::Context;

use crate::CARGO_TOML;

const CARGO_LOCK: &str = "Cargo.lock";

fn cargo_lock_exists(path: &Path) -> bool {
    path.join(CARGO_LOCK).exists()
}

pub fn get_incompatibilities(
    local_package: &Path,
    registry_package: &Path,
) -> anyhow::Result<Option<String>> {
    let local_package_contained_cargo_lock = cargo_lock_exists(local_package);
    let registry_package_contained_cargo_lock = cargo_lock_exists(local_package);

    let output = Command::new("cargo-semver-checks")
        .args(["semver-checks", "check-release"])
        .arg("--manifest-path")
        .arg(&local_package.join(CARGO_TOML))
        .arg("--baseline-root")
        .arg(&registry_package.join(CARGO_TOML))
        .output()
        .with_context(|| format!("error while running cargo-semver-checks on {local_package:?}"))?;

    // Delete Cargo.lock file if cargo-semver-checks created it.
    if !local_package_contained_cargo_lock && cargo_lock_exists(local_package) {
        std::fs::remove_file(local_package.join(CARGO_LOCK))?;
    }
    if !registry_package_contained_cargo_lock && cargo_lock_exists(registry_package) {
        std::fs::remove_file(registry_package.join(CARGO_LOCK))?;
    }

    if output.status.success() {
        Ok(None)
    } else {
        let stderr = String::from_utf8(output.stderr)?;
        if !stderr.is_empty() {
            anyhow::bail!(stderr);
        }
        let stdout = String::from_utf8(output.stdout)?;
        if stdout.is_empty() {
            anyhow::bail!("cargo-semver-checks returned an error but no stderr and stdout");
        }

        Ok(Some(stdout))
    }
}
