use release_plz_core::fs_utils::Utf8TempDir;

use crate::helpers::test_context::TestContext;

#[tokio::test]
#[cfg_attr(not(feature = "docker-tests"), ignore)]
async fn release_info_contains_prs_in_changelog() {
    let context = TestContext::new().await;

    let changelog = r#"
# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.6.5](https://github.com/release-plz/release-plz/compare/git_cmd-v0.6.4...git_cmd-v0.6.5) - 2024-05-05

### Other
- add clippy lints ([#1439](https://github.com/release-plz/release-plz/pull/1439))
- improve uncommitted changes error message ([#1434](https://github.com/release-plz/release-plz/pull/1434))
"#;
    context.write_changelog(changelog);

    let crate_name = &context.gitea.repo;

    let outcome = context.run_release().success();
    let expected_stdout = serde_json::json!({
        "releases": [
            {
                "package_name": crate_name,
                "tag": "v0.1.0",
                "version": "0.1.0",
                "prs": [
                    {
                        "html_url":"https://github.com/release-plz/release-plz/pull/1439",
                        "number":1439
                    },
                    {
                        "html_url":"https://github.com/release-plz/release-plz/pull/1434",
                        "number":1434
                    }
                ],
            }
        ]
    })
    .to_string();
    outcome.stdout(format!("{expected_stdout}\n"));
}

#[tokio::test]
#[cfg_attr(not(feature = "docker-tests"), ignore)]
async fn release_plz_releases_a_new_project_with_custom_tag_name() {
    let context = TestContext::new().await;

    let config = r#"
    [workspace]
    git_tag_name = "{{ package }}--{{ version }}"
    "#;
    context.write_release_plz_toml(config);

    let crate_name = &context.gitea.repo;

    let expected_tag = format!("{crate_name}--0.1.0");
    let is_tag_created = || context.repo.tag_exists(&expected_tag).unwrap();

    assert!(!is_tag_created());

    let outcome = context.run_release().success();
    let expected_stdout = serde_json::json!({
        "releases": [
            {
                "package_name": crate_name,
                "prs": [],
                "tag": expected_tag,
                "version": "0.1.0",
            }
        ]
    })
    .to_string();
    outcome.stdout(format!("{expected_stdout}\n"));

    assert!(is_tag_created());
}

#[tokio::test]
#[cfg_attr(not(feature = "docker-tests"), ignore)]
async fn release_plz_does_not_release_a_new_project_if_release_always_is_false() {
    let context = TestContext::new().await;

    let config = r#"
    [workspace]
    release_always = false
    "#;
    context.write_release_plz_toml(config);

    // Running `release` doesn't release the project
    // because the last commit doesn't belong to a release PR.
    let outcome = context.run_release().success();
    outcome.stdout("{\"releases\":[]}\n");

    let dest_dir = Utf8TempDir::new().unwrap();
    let packages = || context.download_package(dest_dir.path());
    assert!(packages().is_empty());

    // TODO: Gitea doesn't detect associated PRs. I don't know why.
    // context.run_release_pr().success();
    // context.merge_release_pr().unwrap();

    // // Running `release` releases the project
    // // because the last commit belongs to a release PR.
    // let outcome = context.run_release().success();
    // outcome.success();
    // assert_eq!(packages().len(), 1);
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
    let expected_release = format!("{crate_name}--0.1.0");

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
    context.merge_release_pr().await;

    let crate_name = &context.gitea.repo;

    let expected_tag = "v0.1.0";
    let expected_release = format!("{crate_name}--0.1.0");

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

#[tokio::test]
#[cfg_attr(not(feature = "docker-tests"), ignore)]
async fn release_plz_does_not_releases_twice() {
    let context = TestContext::new().await;

    let crate_name = &context.gitea.repo;

    // Running `release` the first time, releases the project
    let outcome = context.run_release().success();
    let expected_stdout = serde_json::json!({
        "releases": [
            {
                "package_name": crate_name,
                "prs": [],
                "tag": "v0.1.0",
                "version": "0.1.0",
            }
        ]
    })
    .to_string();
    outcome.stdout(format!("{expected_stdout}\n"));

    // Running `release` the second time, releases nothing.
    let outcome = context.run_release().success();
    outcome.stdout("{\"releases\":[]}\n");
}
