use crate::helpers::gitea;
use release_plz_core::{Gitea, RepoUrl};
use secrecy::ExposeSecret;

async fn setup(username: String, repo_name: String) -> Gitea {
    let token = gitea::create_user(&username).await;
    gitea::create_repo(&token, &repo_name).await;
    let url: String = format!("{}/{}/{}", gitea::base_url(), username, repo_name);
    Gitea::new(RepoUrl::new(&url).unwrap(), token.clone().into()).unwrap()
}

#[tokio::test]
async fn gitea_client_creates_pr() {
    //TODO testing src/gitea_client.rs
    let user = setup("me".into(), "test_repo".into()).await;
    let branch_for_pr = "test_branch";
    gitea::create_branch(
        user.token.expose_secret(),
        &user.repo,
        &user.owner,
        branch_for_pr,
    )
    .await;

    // TODO create pr
    // TODO check if pr exists
}
