use crate::helpers::comparison_test::ComparisonTest;

#[tokio::test]
async fn up_to_date_project_should_not_raise_pr() {
    let comparison_test = ComparisonTest::new().await;
    comparison_test
        .github_mock_server()
        .expect_no_created_prs()
        .await;
    comparison_test.github_mock_server().no_prs().await;
    comparison_test.open_release_pr().await.unwrap();
}
