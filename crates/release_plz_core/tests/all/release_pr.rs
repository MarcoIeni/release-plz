use crate::helpers::{comparison_test::ComparisonTest, github_mock_server::GitHubMockServer};

#[tokio::test]
async fn up_to_date_project_should_not_raise_pr() {
    let comparison_test = ComparisonTest::new();
    let github_mock_server = GitHubMockServer::start().await;
    comparison_test
        .open_release_pr(github_mock_server.base_url())
        .await
        .unwrap();
    github_mock_server.radio_silence().await;
}
