use cargo_utils::LocalManifest;

use crate::helpers::test_context::TestContext;

#[tokio::test]
#[cfg_attr(not(feature = "docker-tests"), ignore)]
async fn release_plz_detects_edited_readme_cargo_toml_field() {
    let context = TestContext::new().await;

    context.run_release_pr().success();
    context.merge_release_pr().await;

    let expected_tag = "v0.1.0";

    context.run_release().success();

    let gitea_release = context.gitea.get_gitea_release(expected_tag).await;
    assert_eq!(gitea_release.name, expected_tag);

    move_readme(&context, "move readme");

    context.run_release_pr().success();
    context.merge_release_pr().await;

    let expected_tag = "v0.1.1";

    context.run_release().success();

    let gitea_release = context.gitea.get_gitea_release(expected_tag).await;
    assert_eq!(gitea_release.name, expected_tag);
    expect_test::expect![[r#"
        ### Other

        - move readme"#]]
    .assert_eq(&gitea_release.body);
}

#[tokio::test]
#[cfg_attr(not(feature = "docker-tests"), ignore)]
async fn release_plz_honors_features_always_increment_minor_flag() {
    let context = TestContext::new().await;

    let config = r#"
    [workspace]
    features_always_increment_minor = true
    "#;
    context.write_release_plz_toml(config);

    context.run_release_pr().success();
    context.merge_release_pr().await;

    let expected_tag = "v0.1.0";

    context.run_release().success();

    let gitea_release = context.gitea.get_gitea_release(expected_tag).await;
    assert_eq!(gitea_release.name, expected_tag);

    move_readme(&context, "feat: move readme");

    context.run_release_pr().success();
    context.merge_release_pr().await;

    let expected_tag = "v0.2.0";

    context.run_release().success();

    let gitea_release = context.gitea.get_gitea_release(expected_tag).await;
    assert_eq!(gitea_release.name, expected_tag);
    expect_test::expect![[r#"
        ### Added

        - move readme"#]]
    .assert_eq(&gitea_release.body);
}

fn move_readme(context: &TestContext, message: &str) {
    let readme = "README.md";
    let new_readme = format!("NEW_{readme}");
    let old_readme_path = context.repo_dir().join(readme);
    let new_readme_path = context.repo_dir().join(&new_readme);
    fs_err::rename(old_readme_path, new_readme_path).unwrap();

    let cargo_toml_path = context.repo_dir().join("Cargo.toml");
    let mut cargo_toml = LocalManifest::try_new(&cargo_toml_path).unwrap();
    cargo_toml.data["package"]["readme"] = toml_edit::value(new_readme);
    cargo_toml.write().unwrap();

    context.repo.add_all_and_commit(message).unwrap();
    context.repo.git(&["push"]).unwrap();
}
