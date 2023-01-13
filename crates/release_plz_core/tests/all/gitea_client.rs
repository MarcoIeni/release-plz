use crate::helpers::gitea;

#[tokio::test]
async fn create_user_token() {
        gitea::create_user().await;
}
