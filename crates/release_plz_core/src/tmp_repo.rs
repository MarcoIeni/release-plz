use cargo_metadata::camino::Utf8Path;
use git_cmd::Repo;
use tempfile::TempDir;

pub struct TempRepo {
    // temporary directory that will be deleted in the `Drop` method
    _temp_dir: TempDir,
    pub repo: Repo,
}

impl TempRepo {
    pub fn new(temp_dir: TempDir, directory: impl AsRef<Utf8Path>) -> anyhow::Result<Self> {
        Ok(Self {
            _temp_dir: temp_dir,
            repo: Repo::new(directory)?,
        })
    }
}
