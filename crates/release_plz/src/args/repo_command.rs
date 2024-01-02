use std::path::{Path, PathBuf};

use git_cmd::Repo;
use release_plz_core::RepoUrl;

use crate::config::Config;

/// Command that acts on a repo.
pub trait RepoCommand {
    fn optional_project_manifest(&self) -> Option<&Path>;

    fn repo_url(&self) -> Option<&str>;

    fn project_manifest(&self) -> PathBuf {
        super::local_manifest(self.optional_project_manifest())
    }

    fn cargo_metadata(&self) -> anyhow::Result<cargo_metadata::Metadata> {
        cargo_utils::get_manifest_metadata(&self.project_manifest())
    }

    fn get_repo_url(&self, config: &Config) -> anyhow::Result<RepoUrl> {
        match &self.user_repo_url(config) {
            Some(url) => RepoUrl::new(url),
            None => {
                let project_manifest = self.project_manifest();
                let project_dir = release_plz_core::manifest_dir(&project_manifest)?;
                let repo = Repo::new(project_dir)?;
                RepoUrl::from_repo(&repo)
            }
        }
    }

    /// Repo url specified by user
    fn user_repo_url<'a>(&'a self, config: &'a Config) -> Option<&str> {
        self.repo_url().or_else(|| {
            config
                .workspace
                .repo_url
                .as_ref()
                .map(|u| u.as_str())
        })
    }
}
