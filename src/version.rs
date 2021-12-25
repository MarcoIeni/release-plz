use cargo_metadata::Version;

use crate::Diff;

trait NextVersion {
    fn next(self, diff: Diff) -> Self;
}

impl NextVersion for Version {
    fn next(self, diff: Diff) -> Self {
        if !diff.remote_crate_exists {
            self
        } else {
            todo!()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn next_version_of_new_package_is_unchanged() {
        let remote_crate_exists = false;
        let diff = Diff::new(remote_crate_exists);
        let version = Version::new(1, 2, 3);
        assert_eq!(version.clone().next(diff), version);
    }
}
