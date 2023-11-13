use crate::helpers::{test_context::TestContext, TEST_REGISTRY};

#[tokio::test]
#[cfg_attr(not(feature = "docker-tests"), ignore)]
async fn release_plz_adds_changelog_on_new_project() {
    let context = TestContext::new().await;

    context.run_release_pr().success();

    let opened_prs = context.opened_release_prs().await;
    assert_eq!(opened_prs.len(), 1);

    let changed_files = context
        .gitea
        .changed_files_in_pr(opened_prs[0].number)
        .await;
    assert_eq!(changed_files.len(), 1);
    assert_eq!(changed_files[0].filename, "CHANGELOG.md");
}

#[tokio::test]
#[cfg_attr(not(feature = "docker-tests"), ignore)]
async fn release_plz_releases_a_new_project() {
    let context = TestContext::new().await;

    let crate_name = &context.gitea.repo;
    let dest_dir = tempfile::tempdir().unwrap();
    let dest_dir_str = dest_dir.path().to_str().unwrap();

    let packages = || {
        release_plz_core::PackageDownloader::new([crate_name], dest_dir_str)
            .with_registry(TEST_REGISTRY.to_string())
            .with_cargo_cwd(context.repo_dir())
            .download()
            .unwrap()
    };
    // Before running release-plz, no packages should be present.
    assert!(packages().is_empty());

    context.run_release().success();

    assert_eq!(packages().len(), 1);
}
