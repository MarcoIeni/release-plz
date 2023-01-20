use crate::helpers::gitea;
use release_plz_core::{Gitea, RepoUrl};

struct User {
    username: String,
    token: String,
    repo_name: String,
}

impl User {
    fn to_gitea(&self) -> Gitea {
        let url: String = format!("{}/{}/{}", gitea::base_url(), self.username, self.repo_name);
        Gitea::new(RepoUrl::new(&url).unwrap(), self.token.clone().into()).unwrap()
    }
}

async fn setup(username: String, repo_name: String) -> User {
    let token = gitea::create_user(&username).await;
    gitea::create_repo(&token, &repo_name).await;
    User {
        username,
        token,
        repo_name,
    }
}

#[tokio::test]
async fn gitea_client_creates_pr() {
    //TODO testing src/gitea_client.rs
    let user = setup("me".into(), "test_repo".into()).await;
    let branch_for_pr = "test_branch";
    gitea::create_branch(&user.token, &user.repo_name, &user.username, branch_for_pr).await;

    // TODO create pr
    // TODO check if pr exists
}
