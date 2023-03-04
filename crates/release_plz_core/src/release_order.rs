use cargo_metadata::Package;

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

/// Return true if the package is part of a packages array.
/// This function exists because `package.contains(pkg)` is expensive,
/// because it compares the whole package struct.
fn is_package_in(pkg: &Package, packages: &[&Package]) -> bool {
    packages.iter().any(|p| p.name == pkg.name)
}

fn _release_order<'a>(
    packages: &[&'a Package],
    pkg: &'a Package,
    visited: &mut Vec<&'a Package>,
    passed: &mut Vec<&'a Package>,
) -> anyhow::Result<()> {
    if is_package_in(pkg, visited) {
        return Ok(());
    }
    passed.push(pkg);

    for d in &pkg.dependencies {
        // Check if the dependency is part of the packages we are releasing.
        if let Some(dep) = packages
            .iter()
            .find(|p| d.name == p.name && p.name != pkg.name)
        {
            anyhow::ensure!(
                !is_package_in(dep, passed),
                "Circular dependency detected: {} -> {}",
                dep.name,
                pkg.name,
            );
            _release_order(packages, dep, visited, passed)?;
        }
    }

    visited.push(pkg);
    passed.clear();
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

    #[test]
    fn single_package_is_returned() {
        let p: Package = pkg("aaa", &[dep("bbb")]);
        let pkgs = vec![&p];
        let ordered = release_order(&pkgs).unwrap();
        assert_eq!(ordered[0].name, "aaa");
    }

    #[test]
    fn two_packages_cycle_is_detected() {
        let aaa: Package = pkg("aaa", &[dep("bbb")]);
        let bbb: Package = pkg("bbb", &[dep("aaa")]);
        let pkgs = vec![&aaa, &bbb];
        release_order(&pkgs).unwrap_err();
    }

    #[test]
    fn three_packages_cycle_is_detected() {
        let aaa: Package = pkg("aaa", &[dep("bbb")]);
        let ccc: Package = pkg("ccc", &[dep("bbb")]);
        let bbb: Package = pkg("bbb", &[dep("aaa")]);
        let pkgs = vec![&aaa, &bbb, &ccc];
        release_order(&pkgs).unwrap_err();
    }
}
