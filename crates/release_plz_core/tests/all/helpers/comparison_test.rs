use std::{fs, path::PathBuf};

use release_plz_core::{
    are_packages_equal, copy_to_temp_dir, GitHub, ReleasePrRequest, UpdateRequest, CARGO_TOML,
};
use secrecy::Secret;
use tempfile::{tempdir, TempDir};
use url::Url;

/// Compare local project with the one in cargo registry
pub struct ComparisonTest {
    local_project: TempDir,
    registry_project: TempDir,
}

const PROJECT_NAME: &str = "myproject";

impl ComparisonTest {
    pub fn new() -> Self {
        test_logs::init();
        let local_project_dir = tempdir().unwrap();
        let local_project = local_project_dir.as_ref().join(PROJECT_NAME);
        crate::init_project(&local_project);

        let registry_project = copy_to_temp_dir(&local_project).unwrap();
        let comparison = Self {
            local_project: local_project_dir,
            registry_project,
        };
        fs::copy(
            comparison.registry_project().join(CARGO_TOML),
            comparison.registry_project().join("Cargo.toml.orig"),
        )
        .unwrap();
        comparison
    }

    fn update_request(&self) -> UpdateRequest {
        UpdateRequest::new(self.local_project_manifest())
            .unwrap()
            .with_registry_project_manifest(self.registry_project_manfifest())
            .unwrap()
    }

    pub fn run_update(&self) {
        let update_request = self.update_request();
        release_plz_core::update(&update_request).unwrap();
    }

    fn release_pr_request(&self, base_url: Url) -> ReleasePrRequest {
        let github = GitHub::new(
            OWNER.to_string(),
            REPO.to_string(),
            Secret::from("token".to_string()),
        )
        .with_base_url(base_url);
        ReleasePrRequest {
            github,
            update_request: self.update_request(),
        }
    }

    pub async fn open_release_pr(&self, base_url: Url) -> anyhow::Result<()> {
        let release_pr_request = self.release_pr_request(base_url);
        release_plz_core::release_pr(&release_pr_request).await
    }

    pub fn local_project(&self) -> PathBuf {
        self.local_project.as_ref().join(PROJECT_NAME)
    }

    fn registry_project(&self) -> PathBuf {
        self.registry_project.as_ref().join(PROJECT_NAME)
    }

    pub fn local_project_manifest(&self) -> PathBuf {
        self.local_project().join(CARGO_TOML)
    }

    pub fn registry_project_manfifest(&self) -> PathBuf {
        self.registry_project().join(CARGO_TOML)
    }

    pub fn are_projects_equal(&self) -> bool {
        are_packages_equal(&self.local_project(), &self.registry_project())
    }
}

pub const OWNER: &str = "owner";
pub const REPO: &str = "repo";
