use std::path::PathBuf;

use fs_extra::dir;
use release_plz_core::{are_packages_equal, UpdateRequest, CARGO_TOML};
use tempfile::{tempdir, TempDir};

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

    pub fn run_update(&self) {
        let update_request = UpdateRequest::new(self.local_project_manifest())
            .unwrap()
            .with_remote_manifest(self.remote_project_manfifest())
            .unwrap();
        release_plz_core::update(&update_request).unwrap();
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
