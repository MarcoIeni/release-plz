use std::{fs, path::Path};

use camino::Utf8Path;
use tracing::{debug, instrument};

use crate::{git_in_dir, Repo};

impl Repo {
    #[instrument(skip(directory))]
    pub fn init(directory: impl AsRef<Path>) -> Self {
        let directory = Utf8Path::from_path(directory.as_ref()).unwrap();
        git_in_dir(directory, &["init"]).unwrap();

        // configure author
        git_in_dir(directory, &["config", "user.name", "author_name"]).unwrap();
        git_in_dir(directory, &["config", "user.email", "author@example.com"]).unwrap();

        fs::write(directory.join("README.md"), "# my awesome project").unwrap();
        git_in_dir(directory, &["add", "."]).unwrap();
        git_in_dir(directory, &["commit", "-m", "add README"]).unwrap();
        debug!("repo initialized at {:?}", directory);
        Self::new(directory).unwrap()
    }
}
