use release_plz_core::fs_utils::Utf8TempDir;

use crate::helpers::test_context::TestContext;

#[tokio::test]
#[cfg_attr(not(feature = "docker-tests"), ignore)]
async fn release_plz_does_not_open_release_pr_if_there_are_no_release_commits() {
    let context = TestContext::new().await;

    let config = r#"
    [workspace]
    release_commits = "^feat:"
    "#;
    context.write_release_plz_toml(config);

    let outcome = context.run_release_pr().success();
    outcome.stdout("{\"prs\":[]}\n");

    let opened_prs = context.opened_release_prs().await;
    // no features are present in the commits, so release-plz doesn't open the release PR
    assert_eq!(opened_prs.len(), 0);

    fs_err::write(context.repo_dir().join("new.rs"), "// hi").unwrap();
    context.repo.add_all_and_commit("feat: new file").unwrap();

    context.run_release_pr().success();

    // we added a feature, so release-plz opened the release PR
    let opened_prs = context.opened_release_prs().await;
    assert_eq!(opened_prs.len(), 1);
}

#[tokio::test]
#[cfg_attr(not(feature = "docker-tests"), ignore)]
async fn release_plz_adds_changelog_on_new_project() {
    let context = TestContext::new().await;

    let outcome = context.run_release_pr().success();

    let opened_prs = context.opened_release_prs().await;
    assert_eq!(opened_prs.len(), 1);
    let opened_pr = &opened_prs[0];

    let expected_stdout = serde_json::json!({
        "prs": [
          {
            "head_branch": opened_pr.branch(),
            "base_branch": "main",
            "html_url": opened_pr.html_url,
            "number": opened_pr.number
          }
        ]
    })
    .to_string();

    outcome.stdout(format!("{expected_stdout}\n"));

    let changed_files = context.gitea.changed_files_in_pr(opened_pr.number).await;
    assert_eq!(changed_files.len(), 1);
    assert_eq!(changed_files[0].filename, "CHANGELOG.md");
}

#[tokio::test]
#[cfg_attr(not(feature = "docker-tests"), ignore)]
async fn release_plz_releases_a_new_project() {
    let context = TestContext::new().await;

    let dest_dir = Utf8TempDir::new().unwrap();

    let packages = || context.download_package(dest_dir.path());
    // Before running release-plz, no packages should be present.
    assert!(packages().is_empty());

    context.run_release().success();

    assert_eq!(packages().len(), 1);
}

#[tokio::test]
#[cfg_attr(not(feature = "docker-tests"), ignore)]
async fn release_plz_adds_custom_changelog() {
    let context = TestContext::new().await;
    let config = r#"
    [changelog]
    header = "Changelog\n\n"
    body = """
    owner: {{ remote.owner }}, repo: {{ remote.repo }}, link: {{ remote.link }}

    == {{ package }} - [{{ version }}]({{ release_link }})

    {% for group, commits in commits | group_by(attribute="group") %}
    === {{ group | upper_first }}
    {% for commit in commits %}
    {%- if commit.scope -%}
    - *({{commit.scope}})* {% if commit.breaking %}[**breaking**] {% endif %}{{ commit.message }}{%- if commit.links %} ({% for link in commit.links %}[{{link.text}}]({{link.href}}) {% endfor -%}){% endif %}
    {% else -%}
    - {% if commit.breaking %}[**breaking**] {% endif %}{{ commit.message }} by {{ commit.author.name }}
    {% endif -%}
    {% endfor -%}
    {% endfor %}
    """
    trim = true
    "#;
    context.write_release_plz_toml(config);

    context.run_release_pr().success();

    let opened_prs = context.opened_release_prs().await;
    assert_eq!(opened_prs.len(), 1);

    let changelog = context
        .gitea
        .get_file_content(opened_prs[0].branch(), "CHANGELOG.md")
        .await;
    let expected_changelog = "Changelog\n\n";
    let username = context.gitea.user.username();
    let repo = context.gitea.repo;
    let remote_string =
        format!("owner: {username}, repo: {repo}, link: https://localhost/{username}/{repo}\n\n",);
    let package_string =
        format!("== {repo} - [0.1.0](https://localhost/{username}/{repo}/releases/tag/v0.1.0)\n\n");
    let commits = ["add config file", "cargo init", "Initial commit"];
    #[allow(clippy::format_collect)]
    let commits_str = commits
        .iter()
        .map(|commit| format!("- {commit} by {username}\n"))
        .collect::<String>();
    let changes = format!(
        "
=== Other
{commits_str}
"
    );

    let expected_changelog =
        format!("{expected_changelog}{remote_string}{package_string}{changes}");
    assert_eq!(expected_changelog, changelog);
}

#[tokio::test]
#[cfg_attr(not(feature = "docker-tests"), ignore)]
async fn can_generate_single_changelog_for_multiple_packages_in_pr() {
    let context = TestContext::new_workspace(&["crates/one", "crates/two"]).await;
    let config = r#"
    [workspace]
    changelog_path = "./CHANGELOG.md"

    [changelog]
    body = """

    ## `{{ package }}` - [{{ version | trim_start_matches(pat="v") }}](https://github.com/me/my-proj/{% if previous.version %}compare/{{ package }}-v{{ previous.version }}...{{ package }}-v{{ version }}{% else %}releases/tag/{{ package }}-v{{ version }}{% endif %})
    {% for group, commits in commits | group_by(attribute="group") %}
    ### {{ group | upper_first }}
    {% for commit in commits %}
    {%- if commit.scope -%}
    - *({{commit.scope}})* {% if commit.breaking %}[**breaking**] {% endif %}{{ commit.message }}{%- if commit.links %} ({% for link in commit.links %}[{{link.text}}]({{link.href}}) {% endfor -%}){% endif %}
    {% else -%}
    - {% if commit.breaking %}[**breaking**] {% endif %}{{ commit.message }}
    {% endif -%}
    {% endfor -%}
    {% endfor -%}
    """
    "#;
    context.write_release_plz_toml(config);

    context.run_release_pr().success();

    let opened_prs = context.opened_release_prs().await;
    assert_eq!(opened_prs.len(), 1);

    let changelog = context
        .gitea
        .get_file_content(opened_prs[0].branch(), "CHANGELOG.md")
        .await;
    expect_test::expect![[r#"
        # Changelog
        All notable changes to this project will be documented in this file.

        The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
        and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

        ## [Unreleased]

        ## `two` - [0.1.0](https://github.com/me/my-proj/releases/tag/two-v0.1.0)

        ### Other
        - cargo init

        ## `one` - [0.1.0](https://github.com/me/my-proj/releases/tag/one-v0.1.0)

        ### Other
        - cargo init
    "#]]
    .assert_eq(&changelog);
}

#[tokio::test]
#[cfg_attr(not(feature = "docker-tests"), ignore)]
async fn can_generate_single_changelog_for_multiple_packages_locally() {
    let context = TestContext::new_workspace(&["crates/one", "crates/two"]).await;
    let config = r#"
    [workspace]
    changelog_path = "./CHANGELOG.md"

    [changelog]
    body = """

    ## `{{ package }}` - [{{ version | trim_start_matches(pat="v") }}](https://github.com/me/my-proj/{% if previous.version %}compare/{{ package }}-v{{ previous.version }}...{{ package }}-v{{ version }}{% else %}releases/tag/{{ package }}-v{{ version }}{% endif %})
    {% for group, commits in commits | group_by(attribute="group") %}
    ### {{ group | upper_first }}
    {% for commit in commits %}
    {%- if commit.scope -%}
    - *({{commit.scope}})* {% if commit.breaking %}[**breaking**] {% endif %}{{ commit.message }}{%- if commit.links %} ({% for link in commit.links %}[{{link.text}}]({{link.href}}) {% endfor -%}){% endif %}
    {% else -%}
    - {% if commit.breaking %}[**breaking**] {% endif %}{{ commit.message }}
    {% endif -%}
    {% endfor -%}
    {% endfor -%}"""
    "#;
    context.write_release_plz_toml(config);

    context.run_update().success();

    let changelog = fs_err::read_to_string(context.repo.directory().join("CHANGELOG.md")).unwrap();

    expect_test::expect![[r#"
        # Changelog
        All notable changes to this project will be documented in this file.

        The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
        and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

        ## [Unreleased]

        ## `two` - [0.1.0](https://github.com/me/my-proj/releases/tag/two-v0.1.0)

        ### Other
        - cargo init

        ## `one` - [0.1.0](https://github.com/me/my-proj/releases/tag/one-v0.1.0)

        ### Other
        - cargo init
    "#]]
    .assert_eq(&changelog);
}
