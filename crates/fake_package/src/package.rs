use std::path::PathBuf;

use cargo_metadata::{Dependency, Package};
use cargo_utils::{LocalManifest, LocalPackage, Manifest};

use crate::dependency::FakeDependency;

#[derive(Clone, Debug)]
pub struct FakePackage {
    name: String,
    dependencies: Vec<FakeDependency>,
}

impl FakePackage {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            dependencies: vec![],
        }
    }

    pub fn with_dependencies(self, dependencies: Vec<FakeDependency>) -> Self {
        Self {
            dependencies,
            ..self
        }
    }
}

impl From<FakePackage> for Package {
    fn from(pkg: FakePackage) -> Self {
        let dependencies: Vec<Dependency> =
            pkg.dependencies.into_iter().map(Dependency::from).collect();
        let name = pkg.name;
        serde_json::from_value(serde_json::json!({
            "name": name,
            "version": "0.1.0",
            "id": name,
            "dependencies": dependencies,
            "features": {},
            "manifest_path": format!("{name}/Cargo.toml"),
            "targets": [],
        }))
        .unwrap()
    }
}

impl From<FakePackage> for LocalPackage {
    #[allow(clippy::default_trait_access)]
    fn from(pkg: FakePackage) -> Self {
        let package = Package::from(pkg);
        Self {
            package,
            manifest: LocalManifest {
                path: PathBuf::from("Cargo.toml"),
                manifest: Manifest {
                    data: Default::default(),
                },
            },
        }
    }
}
