use cargo_metadata::camino::{Utf8Path, Utf8PathBuf};
use git_cmd::Repo;
use release_plz_core::RepoUrl;

use crate::config::Config;

/// Command that acts on a repo.
pub trait RepoCommand {
    fn optional_project_manifest(&self) -> Option<&Utf8Path>;

    fn repo_url(&self) -> Option<&str>;

    fn project_manifest(&self) -> Utf8PathBuf {
        super::local_manifest(self.optional_project_manifest())
    }

    fn cargo_metadata(&self) -> anyhow::Result<cargo_metadata::Metadata> {
        let manifest = &self.project_manifest();
        cargo_utils::get_manifest_metadata(manifest).map_err(|e| match e {
            cargo_metadata::Error::CargoMetadata { stderr } => {
                let stderr = stderr.trim();
                anyhow::anyhow!("{stderr}. Use --project-manifest to specify the path to the manifest file if it's not in the current directory.")
            }
            _ => {
                anyhow::anyhow!(e)
            }
        })
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
        self.repo_url()
            .or_else(|| config.workspace.repo_url.as_ref().map(|u| u.as_str()))
    }
}
