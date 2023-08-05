use crate::helpers::test_context::TestContext;

#[tokio::test]
async fn release_plz_adds_changelog_on_new_project() {
    let context = TestContext::new().await;

    context.run_release_plz().success();

    let opened_prs = context.opened_release_prs().await;
    assert_eq!(opened_prs.len(), 1);
}
