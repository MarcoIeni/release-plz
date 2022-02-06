use std::path::PathBuf;

use fs_extra::dir;
use release_plz_core::{are_packages_equal, GitHub, ReleasePrRequest, UpdateRequest, CARGO_TOML};
use secrecy::Secret;
use tempfile::{tempdir, TempDir};
use url::Url;

/// Compare local project with remote one
pub struct ComparisonTest {
    local_project: TempDir,
    remote_project: TempDir,
}

const PROJECT_NAME: &str = "myproject";

impl ComparisonTest {
    pub fn new() -> Self {
        test_logs::init();
        let local_project_dir = tempdir().unwrap();
        let local_project = local_project_dir.as_ref().join(PROJECT_NAME);
        crate::init_project(&local_project);

        let remote_project = tempdir().unwrap();
        dir::copy(
            &local_project,
            remote_project.as_ref(),
            &dir::CopyOptions::default(),
        )
        .unwrap();
        Self {
            local_project: local_project_dir,
            remote_project,
        }
    }

    fn update_request(&self) -> UpdateRequest {
        UpdateRequest::new(self.local_project_manifest())
            .unwrap()
            .with_remote_manifest(self.remote_project_manfifest())
            .unwrap()
    }

    pub fn run_update(&self) {
        let update_request = self.update_request();
        release_plz_core::update(&update_request).unwrap();
    }

    fn release_pr_request(&self, base_url: Url) -> ReleasePrRequest {
        let github = GitHub::new(
            "owner".to_string(),
            "repo".to_string(),
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

    fn remote_project(&self) -> PathBuf {
        self.remote_project.as_ref().join(PROJECT_NAME)
    }

    pub fn local_project_manifest(&self) -> PathBuf {
        self.local_project().join(CARGO_TOML)
    }

    pub fn remote_project_manfifest(&self) -> PathBuf {
        self.remote_project().join(CARGO_TOML)
    }

    pub fn are_projects_equal(&self) -> bool {
        are_packages_equal(&self.local_project(), &self.remote_project())
    }
}
