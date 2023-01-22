// Copied from [cargo-clone](https://github.com/JanLikar/cargo-clone/blob/89ba4da215663ffb3b8c93a674f3002937eafec4/cargo-clone-core/src/cloner_builder.rs)

use std::{env, path::PathBuf};

use anyhow::Context;
use cargo::{CargoResult, Config};

use super::{Cloner, ClonerSource};

/// Builder for [`Cloner`].
#[derive(Debug, Default)]
pub struct ClonerBuilder {
    config: Option<Config>,
    directory: Option<PathBuf>,
    source: ClonerSource,
    use_git: bool,
}

impl ClonerBuilder {
    /// Creates a new [`ClonerBuilder`] that:
    /// - Uses crates.io as source.
    /// - Clones the crates into the current directory.
    pub fn new() -> Self {
        Self::default()
    }

    /// Clone into a different directory, instead of the current one.
    pub fn with_directory(self, directory: impl Into<PathBuf>) -> Self {
        Self {
            directory: Some(directory.into()),
            ..self
        }
    }

    /// Clone from an alternative source, instead of crates.io.
    pub fn with_source(self, source: ClonerSource) -> Self {
        Self { source, ..self }
    }

    /// Build the [`Cloner`].
    pub fn build(self) -> CargoResult<Cloner> {
        let config = match self.config {
            Some(config) => config,
            None => Config::default().context("Unable to get cargo config.")?,
        };

        let directory = match self.directory {
            Some(directory) => directory,
            None => env::current_dir().context("Unable to get current directory.")?,
        };

        let srcid = self
            .source
            .cargo_source
            .to_source_id(&config)
            .context("can't determine the source id")?;

        Ok(Cloner {
            config,
            directory,
            srcid,
            use_git: self.use_git,
        })
    }
}
