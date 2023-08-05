use crate::helpers::test_context::TestContext;

#[tokio::test]
async fn release_plz_adds_changelog_on_new_project() {
    let context = TestContext::new().await;

    assert_cmd::Command::cargo_bin(env!("CARGO_PKG_NAME"))
        .unwrap()
        .current_dir(&context.repo_dir())
        .env("RUST_LOG", "DEBUG,hyper=info")
        .arg("release-pr")
        .arg("--git-token")
        .arg(&context.gitea.token)
        .arg("--backend")
        .arg("gitea")
        .assert()
        .success();

    let opened_prs = context.opened_release_prs().await;
    assert_eq!(opened_prs.len(), 1);
}
