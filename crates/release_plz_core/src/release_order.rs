use cargo_metadata::{DependencyKind, Package};
use tracing::debug;

/// Return packages in an order they can be released.
/// In the result, the packages are placed after all their dependencies.
/// Return an error if a circular dependency is detected.
pub fn release_order<'a>(packages: &'a [&Package]) -> anyhow::Result<Vec<&'a Package>> {
    let mut order = vec![];
    let mut passed = vec![];
    for p in packages {
        release_order_inner(packages, p, &mut order, &mut passed)?;
    }
    debug!(
        "Release order: {:?}",
        order.iter().map(|p| &p.name).collect::<Vec<_>>()
    );
    Ok(order)
}

/// Return true if the package is part of a packages array.
/// This function exists because `package.contains(pkg)` is expensive,
/// because it compares the whole package struct.
fn is_package_in(pkg: &Package, packages: &[&Package]) -> bool {
    packages.iter().any(|p| p.name == pkg.name)
}

/// The `passed` argument is used to track packages that you already visited to
/// detect circular dependencies.
fn release_order_inner<'a>(
    packages: &[&'a Package],
    pkg: &'a Package,
    order: &mut Vec<&'a Package>,
    passed: &mut Vec<&'a Package>,
) -> anyhow::Result<()> {
    if is_package_in(pkg, order) {
        return Ok(());
    }
    passed.push(pkg);

    for d in &pkg.dependencies {
        // Check if the dependency is part of the packages we are releasing.
        if let Some(dep) = packages.iter().find(|p| {
            d.name == p.name
              // Exclude the current package.
              && p.name != pkg.name
              // Ignore development dependencies. They don't need to be published before.
              && matches!(d.kind, DependencyKind::Normal | DependencyKind::Build)
        }) {
            anyhow::ensure!(
                !is_package_in(dep, passed),
                "Circular dependency detected: {} -> {}",
                dep.name,
                pkg.name,
            );
            release_order_inner(packages, dep, order, passed)?;
        }
    }

    order.push(pkg);
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
        assert_eq!(
            order(pkgs),
            [
                "cargo_utils",
                "git_cmd",
                "test_logs",
                "next_version",
                "release_plz_core",
                "release-plz"
            ]
        );
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
        dependency(name, "normal")
    }

    /// Development dependency
    fn dev_dep(name: &str) -> Dependency {
        dependency(name, "dev")
    }

    fn dependency(name: &str, dev_type: &str) -> Dependency {
        serde_json::from_value(serde_json::json!({
            "name": name,
            "req": "0.1.0",
            "kind": dev_type,
            "optional": false,
            "uses_default_features": true,
            "features": [],
        }))
        .unwrap()
    }

    fn order<'a>(pkgs: &'a [&'a Package]) -> Vec<&'a str> {
        release_order(pkgs)
            .unwrap()
            .iter()
            .map(|p| p.name.as_str())
            .collect()
    }

    // Diagrams created with https://asciiflow.com/

    /// A─►B
    #[test]
    fn single_package_is_returned() {
        let pkgs = [&pkg("a", &[dep("b")])];
        assert_eq!(order(&pkgs), ["a"]);
    }

    /// ┌──┐
    /// │  ▼
    /// A  B
    /// ▲  │
    /// └──┘
    #[test]
    fn two_packages_cycle_is_detected() {
        let pkgs = [&pkg("a", &[dep("b")]), &pkg("b", &[dep("a")])];
        expect_test::expect!["Circular dependency detected: a -> b"]
            .assert_eq(&release_order(&pkgs).unwrap_err().to_string());
    }

    /// ┌──┐
    /// │  ▼
    /// A  B (dev dependency)
    /// ▲  │
    /// └──┘
    #[test]
    fn two_packages_dev_cycle_is_ok() {
        let pkgs = [&pkg("a", &[dev_dep("b")]), &pkg("b", &[dep("a")])];
        assert_eq!(order(&pkgs), ["a", "b"]);

        // check if the order of the vector matters.
        let pkgs = [&pkg("b", &[dep("a")]), &pkg("a", &[dev_dep("b")])];
        assert_eq!(order(&pkgs), ["a", "b"]);
    }

    /// ┌─────┐
    /// ▼     │
    /// A────►B
    /// │     ▲
    /// └─►C──┘
    #[test]
    fn three_packages_cycle_is_detected() {
        let pkgs = [
            &pkg("a", &[dep("b")]),
            &pkg("a", &[dep("c")]),
            &pkg("b", &[dep("a")]),
            &pkg("c", &[dep("b")]),
        ];
        expect_test::expect!["Circular dependency detected: a -> b"]
            .assert_eq(&release_order(&pkgs).unwrap_err().to_string());
    }

    /// A────►C
    /// │     ▲
    /// └─►B──┘
    #[test]
    fn three_packages_are_ordered() {
        let pkgs = [
            &pkg("a", &[dep("b")]),
            &pkg("b", &[dep("c")]),
            &pkg("c", &[]),
        ];
        assert_eq!(order(&pkgs), ["c", "b", "a"]);
    }
}
