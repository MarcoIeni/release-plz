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
async fn release_plz_releases_a_new_project_with_custom_release() {
    let context = TestContext::new().await;

    let config = r#"
    [workspace]
    git_release_name = "{{ package }}--{{ version }}"
    git_release_body = "Welcome to this new release!\n{{ changelog }}"
    "#;
    context.write_release_plz_toml(config);

    let crate_name = &context.gitea.repo;

    let expected_tag = "v0.1.0";
    let expected_release = format!("{}--0.1.0", crate_name);

    context.run_release().success();

    let gitea_release = context.gitea.get_gitea_release(expected_tag).await;
    assert_eq!(gitea_release.name, expected_release);
    // There's no changelog, so {{ changelog }} should be empty
    expect_test::expect!["Welcome to this new release!\n"].assert_eq(&gitea_release.body);
}

#[tokio::test]
#[cfg_attr(not(feature = "docker-tests"), ignore)]
async fn release_plz_releases_after_release_pr_merged() {
    let context = TestContext::new().await;

    let config = r#"
    [workspace]
    git_release_name = "{{ package }}--{{ version }}"
    git_release_body = "Welcome to this new release! Changes:\n\n{{ changelog }}"
    "#;
    context.write_release_plz_toml(config);

    context.run_release_pr().success();

    let opened_prs = context.opened_release_prs().await;
    assert_eq!(opened_prs.len(), 1);
    context.gitea.merge_pr_retrying(opened_prs[0].number).await;
    context.repo.git(&["pull"]).unwrap();

    let crate_name = &context.gitea.repo;

    let expected_tag = "v0.1.0";
    let expected_release = format!("{}--0.1.0", crate_name);

    context.run_release().success();

    let gitea_release = context.gitea.get_gitea_release(expected_tag).await;
    assert_eq!(gitea_release.name, expected_release);
    expect_test::expect![[r#"
        Welcome to this new release! Changes:

        ### Other
        - add config file
        - cargo init
        - Initial commit"#]]
    .assert_eq(&gitea_release.body);
}
