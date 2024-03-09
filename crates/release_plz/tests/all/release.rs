use crate::helpers::test_context::TestContext;

#[tokio::test]
#[cfg_attr(not(feature = "docker-tests"), ignore)]
async fn release_plz_releases_a_new_project_with_custom_tag_name() {
    let context = TestContext::new().await;

    let config = r#"
    [workspace]
    git_tag_name = "{{ package}}--{{ version }}"
    "#;
    context.write_release_plz_toml(config);

    let crate_name = &context.gitea.repo;

    let expected_tag = format!("{}--0.1.0", crate_name);
    let is_tag_created = || context.repo.tag_exists(&expected_tag).unwrap();

    assert!(!is_tag_created());

    context.run_release().success();

    assert!(is_tag_created());
}

#[tokio::test]
#[cfg_attr(not(feature = "docker-tests"), ignore)]
async fn release_plz_releases_a_new_project_with_custom_release_name() {
    let context = TestContext::new().await;

    let config = r#"
    [workspace]
    git_release_name = "{{ package}}--{{ version }}"
    "#;
    context.write_release_plz_toml(config);

    let crate_name = &context.gitea.repo;

    let expected_tag = "v0.1.0";
    let expected_release = format!("{}--0.1.0", crate_name);

    context.run_release().success();

    let gitea_release = context.gitea.get_gitea_release(expected_tag).await;
    assert_eq!(gitea_release.name, expected_release);
}
