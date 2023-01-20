use crate::helpers::gitea;

#[tokio::test]
async fn create_user_token() {
    //TODO testing src/gitea_client.rs
    let token = gitea::create_user().await;
    gitea::create_repo(&token, "my_test").await;
    // TODO create pr
    // TODO check if pr exists
}
