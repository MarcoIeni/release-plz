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
