use std::{fs, path::PathBuf};

use chrono::{TimeZone, Utc};
use release_plz_core::{
    are_packages_equal, copy_to_temp_dir, ChangelogRequest, GitHub, ReleasePrRequest,
    UpdateRequest, CARGO_TOML, CHANGELOG_FILENAME,
};
use secrecy::Secret;
use tempfile::{tempdir, TempDir};
use url::Url;

use super::github_mock_server::GitHubMockServer;

/// Compare local project with the one in cargo registry
pub struct ComparisonTest {
    local_project: TempDir,
    registry_project: TempDir,
    github_mock_server: GitHubMockServer,
}

const PROJECT_NAME: &str = "myproject";
pub const OWNER: &str = "owner";
pub const REPO: &str = "repo";

impl ComparisonTest {
    pub async fn new() -> Self {
        test_logs::init();
        let local_project_dir = tempdir().unwrap();
        let local_project = local_project_dir.as_ref().join(PROJECT_NAME);
        crate::init_project(&local_project);

        let registry_project = copy_to_temp_dir(&local_project).unwrap();
        let comparison = Self {
            local_project: local_project_dir,
            registry_project,
            github_mock_server: GitHubMockServer::start(OWNER, REPO).await,
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
            .with_changelog(ChangelogRequest {
                release_date: Some(Utc.ymd(2015, 5, 15)),
            })
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

    pub async fn open_release_pr(&self) -> anyhow::Result<()> {
        let base_url = self.github_mock_server.base_url();
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

    pub fn write_local_project_changelog(&self, changelog: &str) {
        let changelog_path = self.local_project_changelog_path();
        fs::write(changelog_path, changelog).unwrap()
    }

    pub fn local_project_changelog(&self) -> String {
        let changelog_path = self.local_project_changelog_path();
        fs::read_to_string(changelog_path).unwrap()
    }

    fn local_project_changelog_path(&self) -> PathBuf {
        self.local_project().join(CHANGELOG_FILENAME)
    }

    /// Get a reference to the comparison test's github mock server.
    #[must_use]
    pub fn github_mock_server(&self) -> &GitHubMockServer {
        &self.github_mock_server
    }
}
