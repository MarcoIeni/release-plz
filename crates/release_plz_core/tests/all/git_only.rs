use crate::helpers::git_only_test::{GitOnlyTestContext, PROJECT_NAME};
use cargo_metadata::semver::Version;
use release_plz_core::PackagesUpdate;
use std::path::PathBuf;

fn touch(path: impl Into<PathBuf>) -> std::io::Result<()> {
    fs_err::OpenOptions::new()
        .create(true)
        .write(true)
        .open(path)
        .map(|_| ())
}

trait AssertUpdate {
    fn assert_packages_updated<'s>(
        &self,
        packages: impl IntoIterator<IntoIter: ExactSizeIterator<Item = (&'s str, Version, Version)>>,
    );
}

impl AssertUpdate for PackagesUpdate {
    fn assert_packages_updated<'s>(
        &self,
        packages: impl IntoIterator<IntoIter: ExactSizeIterator<Item = (&'s str, Version, Version)>>,
    ) {
        {
            let packages = packages.into_iter();
            let updates = self.updates();
            assert_eq!(updates.len(), packages.len());

            for ((name, old_version, new_version), (package, result)) in packages.zip(updates) {
                assert_eq!(package.name, name);
                assert_eq!(package.version, old_version);
                assert_eq!(result.version, new_version);
            }
        }
    }
}

#[tokio::test]
async fn single_crate() {
    let context = GitOnlyTestContext::new(None).await;

    context
        .run_update_and_commit()
        .await
        .expect("initial update should succeed")
        .assert_packages_updated([(PROJECT_NAME, Version::new(0, 1, 0), Version::new(0, 1, 0))]);

    context
        .run_release()
        .await
        .expect("initial release should succeed")
        .expect("initial release should not be empty");

    touch(context.project_dir().join("included")).unwrap();
    context.add_all_commit_and_push("fix: Add `included` file");

    // Add package excludes
    const EXCLUDED_FILENAME: &str = "excluded";
    context
        .write_root_cargo_toml(|cargo_toml| {
            cargo_toml["package"]["exclude"] =
                toml_edit::Array::from_iter([EXCLUDED_FILENAME]).into();
        })
        .unwrap();

    context.add_all_commit_and_push("fix: Exclude `excluded` from package");

    context
        .run_update_and_commit()
        .await
        .expect("second update should succeed")
        .assert_packages_updated([(PROJECT_NAME, Version::new(0, 1, 0), Version::new(0, 1, 1))]);

    context
        .run_release()
        .await
        .expect("second release should succeed")
        .expect("second release should not be empty");

    touch(context.project_dir().join(EXCLUDED_FILENAME)).unwrap();
    context.add_all_commit_and_push("chore: Add `excluded` file");

    // Modifying file excluded from package should not lead to version increment
    context
        .run_update()
        .await
        .expect("update should succeed")
        .assert_packages_updated([]);

    assert!(context.repo.is_clean().is_ok());
}

#[tokio::test]
async fn workspace() {
    let (context, crates) = GitOnlyTestContext::new_workspace(
        Some("{{ package }}--vv{{ version }}".into()),
        ["publish-false", "other"],
    )
    .await;
    let [publish_false_crate, other_crate] = &crates;
    let crate_names = crates.each_ref().map(|dir| dir.file_name().unwrap());
    let [publish_false_crate_name, other_crate_name] = crate_names;

    context
        .run_update_and_commit()
        .await
        .expect("initial update should succeed")
        .assert_packages_updated(
            crate_names
                .each_ref()
                .map(|&name| (name, Version::new(0, 1, 0), Version::new(0, 1, 0))),
        );

    context
        .run_release()
        .await
        .expect("initial release should succeed")
        .expect("initial release should not be empty");

    // Write publish = false in Cargo.toml
    context
        .write_cargo_toml(publish_false_crate, |cargo_toml| {
            cargo_toml["package"]["publish"] = false.into();
        })
        .unwrap();
    context.add_all_commit_and_push("fix(publish-false): Set package.publish = false");

    context
        .run_update_and_commit()
        .await
        .expect("publish-false update should succeed")
        .assert_packages_updated([(
            publish_false_crate_name,
            Version::new(0, 1, 0),
            Version::new(0, 1, 1),
        )]);

    context
        .run_release()
        .await
        .expect("publish-false release should succeed")
        .expect("publish-false release should not be empty");

    touch(context.crate_dir(publish_false_crate).join("foo")).unwrap();
    context.add_all_commit_and_push("fix(publish-false): Add foo");

    touch(context.crate_dir(other_crate).join("bar")).unwrap();
    context.add_all_commit_and_push("feat(other)!: Add bar");

    context
        .run_update()
        .await
        .expect("crates update should succeed")
        .assert_packages_updated([
            (
                publish_false_crate_name,
                Version::new(0, 1, 1),
                Version::new(0, 1, 2),
            ),
            (
                other_crate_name,
                Version::new(0, 1, 0),
                Version::new(0, 2, 0),
            ),
        ]);

    // TODO: Check contents of release
    context
        .run_release()
        .await
        .expect("publish-false release should succeed")
        .expect("publish-false release should not be empty");
}
