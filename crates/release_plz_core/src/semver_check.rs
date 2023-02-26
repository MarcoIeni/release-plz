use std::{
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::Context;

use crate::CARGO_TOML;

fn target_dir(path: &Path) -> PathBuf {
    path.join("target")
}

fn cargo_lock(path: &Path) -> PathBuf {
    path.join("Cargo.lock")
}

fn is_cargo_semver_checks_installed() -> bool {
    Command::new("cargo-semver-checks")
        .arg("--version")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

/// Outcome of semver check.
#[derive(Debug)]
pub enum SemverCheck {
    /// Semver check done. No incompatibilities found.
    Compatible,
    /// Semver check done. Incompatibilities found.
    Incompatible(String),
    /// Semver check skipped. This is the expected state for binaries.
    Skipped,
}

impl SemverCheck {
    pub fn outcome_str(&self) -> &'static str {
        match self {
            SemverCheck::Compatible => " (✓ API compatible changes)",
            SemverCheck::Incompatible(_) => " (⚠️ API breaking changes)",
            SemverCheck::Skipped => "",
        }
    }
}

pub fn run_semver_check(
    local_package: &Path,
    registry_package: &Path,
) -> anyhow::Result<SemverCheck> {
    if !is_cargo_semver_checks_installed() {
        return Ok(SemverCheck::Skipped);
    }

    let local_cargo_lock = cargo_lock(local_package);
    let registry_cargo_lock = cargo_lock(registry_package);
    let local_target_dir = target_dir(local_package);
    let registry_target_dir = target_dir(registry_package);

    let local_package_contained_cargo_lock = local_cargo_lock.exists();
    let registry_package_contained_cargo_lock = registry_cargo_lock.exists();
    let local_package_contained_target = local_target_dir.exists();
    let registry_package_contained_target = registry_target_dir.exists();

    let output = Command::new("cargo-semver-checks")
        .args(["semver-checks", "check-release"])
        .arg("--manifest-path")
        .arg(&local_package.join(CARGO_TOML))
        .arg("--baseline-root")
        .arg(&registry_package.join(CARGO_TOML))
        .output()
        .with_context(|| format!("error while running cargo-semver-checks on {local_package:?}"))?;

    // Delete Cargo.lock file if cargo-semver-checks created it.
    if !local_package_contained_cargo_lock && local_cargo_lock.exists() {
        std::fs::remove_file(local_cargo_lock)?;
    }
    if !registry_package_contained_cargo_lock && registry_cargo_lock.exists() {
        std::fs::remove_file(registry_cargo_lock)?;
    }
    // Delete target dir if cargo-semver-checks created it.
    if !local_package_contained_target && local_target_dir.exists() {
        std::fs::remove_dir_all(local_target_dir)?;
    }
    if !registry_package_contained_target && registry_target_dir.exists() {
        std::fs::remove_dir_all(registry_target_dir)?;
    }

    if output.status.success() {
        Ok(SemverCheck::Compatible)
    } else {
        let stderr = String::from_utf8(output.stderr)?;
        if stderr.contains("semver requires new major version") {
            let stdout = String::from_utf8(output.stdout)?;
            if stdout.is_empty() {
                anyhow::bail!("unknown source of semver incompatibility");
            }
            Ok(SemverCheck::Incompatible(stdout))
        } else {
            Ok(SemverCheck::Compatible)
        }
    }
}
