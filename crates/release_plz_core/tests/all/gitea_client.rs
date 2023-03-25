use crate::helpers::gitea;

#[tokio::test]
async fn create_gitea_repository() {
    let user = gitea::create_user();
    let repo_name = "myrepo";
    user.create_repository(repo_name).await;
    assert!(user.repo_exists(repo_name).await);
}
