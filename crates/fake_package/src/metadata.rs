use std::path::PathBuf;

pub fn fake_metadata() -> cargo_metadata::Metadata {
    cargo_utils::get_manifest_metadata(&PathBuf::from("Cargo.toml")).unwrap()
}
