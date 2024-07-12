use std::path::Path;

use anyhow::Context as _;

use crate::config::Config;

pub trait ConfigCommand {
    fn config_path(&self) -> Option<&Path>;

    fn config(&self) -> anyhow::Result<Config> {
        super::parse_config(self.config_path()).context("failed to parse release-plz configuration")
    }
}
