use crate::helpers::test_context::TestContext;

#[tokio::test]
async fn release_plz_adds_changelog_on_new_project() {
    let context = TestContext::new().await;

    context.run_release_plz().success();

    let opened_prs = context.opened_release_prs().await;
    assert_eq!(opened_prs.len(), 1);

    let changed_files = context
        .gitea
        .changed_files_in_pr(opened_prs[0].number)
        .await;
    assert_eq!(changed_files.len(), 1,);
    assert_eq!(changed_files[0].filename, "CHANGELOG.md");
}
