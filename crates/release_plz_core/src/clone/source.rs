// Copied from [cargo-clone](https://github.com/JanLikar/cargo-clone/blob/89ba4da215663ffb3b8c93a674f3002937eafec4/cargo-clone-core/src/source.rs)

use cargo::{core::SourceId, CargoResult, GlobalContext};

/// Where to clone the crate from.
#[derive(Debug, Default, Clone)]
pub struct ClonerSource {
    pub(crate) cargo_source: CargoSource,
}

#[derive(Debug, Default, Clone)]
pub(crate) enum CargoSource {
    #[default]
    CratesIo,
    Registry(String),
}

impl ClonerSource {
    /// Creates a [`ClonerSource`] from the name of the remote registry.
    pub fn registry(key: impl Into<String>) -> Self {
        Self {
            cargo_source: CargoSource::Registry(key.into()),
        }
    }

    /// Creates a [`ClonerSource`] from [crates.io](https://crates.io/).
    pub fn crates_io() -> Self {
        Self {
            cargo_source: CargoSource::CratesIo,
        }
    }
}

impl CargoSource {
    pub(crate) fn to_source_id(&self, config: &GlobalContext) -> CargoResult<SourceId> {
        match self {
            CargoSource::CratesIo => SourceId::crates_io(config),
            CargoSource::Registry(key) => SourceId::alt_registry(config, key),
        }
    }
}
