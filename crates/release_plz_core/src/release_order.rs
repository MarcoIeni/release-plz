use cargo_metadata::{DependencyKind, Package};

/// Return packages in an order they can be released.
/// In the result, the packages are placed after all their dependencies.
/// Return an error if a circular dependency is detected.
pub fn release_order<'a>(packages: &'a [&Package]) -> anyhow::Result<Vec<&'a Package>> {
    let mut visited = vec![];
    let mut passed = vec![];
    for p in packages {
        _release_order(packages, p, &mut visited, &mut passed)?;
    }
    Ok(visited)
}

fn _release_order<'a>(
    packages: &[&'a Package],
    pkg: &'a Package,
    visited: &mut Vec<&'a Package>,
    passed: &mut Vec<&'a Package>,
) -> anyhow::Result<()> {
    // TODO: check for circular dependencies
    passed.push(pkg);
    if visited.contains(&pkg) {
        return Ok(());
    }

    for d in &pkg.dependencies {
        // Check if the dependency is part of the packages we are releasing.
        if let Some(dep) = packages.iter().find(|p| d.name == p.name) {
            match d.kind {
                DependencyKind::Normal | DependencyKind::Build => {
                    if !passed.contains(&dep) {
                        _release_order(packages, dep, visited, passed)?;
                    }
                }
                DependencyKind::Development | DependencyKind::Unknown => {}
            }
        }
    }

    visited.push(pkg);
    Ok(())
}

#[cfg(test)]
mod tests {
    use cargo_metadata::Dependency;

    use super::*;
    use crate::publishable_packages;

    // Test the package release order in the release-plz workspace itself.
    #[test]
    fn workspace_release_order_is_correct() {
        let public_packages = publishable_packages("../../Cargo.toml").unwrap();
        let pkgs = &public_packages.iter().collect::<Vec<_>>();
        let ordered = release_order(pkgs).unwrap();
        assert_eq!(ordered[ordered.len() - 1].name, "release-plz");
        assert_eq!(ordered[ordered.len() - 2].name, "release_plz_core");
    }

    /// Package
    fn pkg(name: &str, deps: &[Dependency]) -> Package {
        serde_json::from_value(serde_json::json!({
            "name": name,
            "version": "0.1.0",
            "id": name,
            "dependencies": deps,
            "features": {},
            "manifest_path": format!("{name}/Cargo.toml"),
            "targets": [],
        }))
        .unwrap()
    }

    /// Dependency
    fn dep(name: &str) -> Dependency {
        serde_json::from_value(serde_json::json!({
            "name": name,
            "req": "0.1.0",
            "kind": "normal",
            "optional": false,
            "uses_default_features": true,
            "features": [],
        }))
        .unwrap()
    }

    // Test the package release order in the release-plz workspace itself.
    #[test]
    fn single_package_is_returned() {
        let p: Package = pkg("aaa", &[dep("bbb")]);
        let pkgs = vec![&p];
        let ordered = release_order(&pkgs).unwrap();
        assert_eq!(ordered[0].name, "aaa");
    }
}
