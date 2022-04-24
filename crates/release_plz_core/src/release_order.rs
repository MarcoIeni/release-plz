use cargo_metadata::{DependencyKind, Package};

/// Return packages in an order they can be released.
/// In the result, the packages are placed after all their dependencies.
pub fn release_order<'a>(packages: &'a [&Package]) -> Vec<&'a Package> {
    let mut visited = vec![];
    for p in packages {
        _release_order(packages, p, &mut visited);
    }
    visited
}

fn _release_order<'a>(packages: &[&'a Package], pkg: &'a Package, visited: &mut Vec<&'a Package>) {
    if visited.contains(&pkg) {
        return;
    }

    for d in &pkg.dependencies {
        if let Some(dep) = packages.iter().find(|p| d.name == p.name) {
            match d.kind {
                DependencyKind::Normal | DependencyKind::Build => {
                    _release_order(packages, dep, visited);
                }
                _ => {}
            }
        }
    }

    visited.push(pkg);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::publishable_packages;

    // Test the package release order in the release-plz workspace itself.
    #[test]
    fn workspace_release_order_is_correct() {
        let public_packages = publishable_packages("../../Cargo.toml").unwrap();
        let pkgs = &public_packages.iter().collect::<Vec<_>>();
        let ordered = release_order(pkgs);
        assert_eq!(ordered[ordered.len() - 1].name, "release-plz");
        assert_eq!(ordered[ordered.len() - 2].name, "release_plz_core");
    }
}
