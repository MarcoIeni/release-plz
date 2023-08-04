use std::{process::Command, str::FromStr};

use git_cmd::Repo;
use release_plz_core::{GitBackend, GitClient, Gitea, RepoUrl};
use secrecy::SecretString;

use crate::helpers::gitea;

#[tokio::test]
async fn create_gitea_repository() {
    let user = gitea::create_user();
    let repo_name = "myrepo";
    user.create_repository(repo_name).await;
    assert!(user.repo_exists(repo_name).await);
}

#[tokio::test]
async fn create_token() {
    let user = gitea::create_user();
    let token = user.create_token().await;
    println!("Token: {}", token);
}

#[tokio::test]
async fn release_plz_adds_changelog_on_new_project() {
    test_logs::init();
    let user = gitea::create_user();
    let repo_name = "myrepo";
    user.create_repository(repo_name).await;
    let temp = tempfile::tempdir().unwrap();
    let token = user.create_token().await;
    let repo_url = format!(
        //"ssh://git@localhost:2222/{}/{}.git",
        "http://{}:{}@localhost:3000/{}/{}.git",
        user.username(),
        user.password(),
        user.username(),
        repo_name
    );

    println!("temp: {:?}", temp.path());

    let result = Command::new("git")
        .current_dir(temp.path())
        .arg("clone")
        .arg(&repo_url)
        .spawn()
        .unwrap()
        .wait()
        .unwrap();
    assert!(result.success());

    let repo_dir = temp.path().join(repo_name);
    let result = Command::new("cargo")
        .current_dir(&repo_dir)
        .arg("init")
        .output()
        .unwrap();
    assert!(result.status.success());

    let repo = Repo::new(&repo_dir).unwrap();
    // config local user
    repo.git(&["config", "user.name", user.username()]).unwrap();
    // set email
    repo.git(&["config", "user.email", "a@example.com"])
        .unwrap();

    repo.add_all_and_commit("Initial commit").unwrap();

    // TODO: git push

    let result = Command::new("release-plz")
        .current_dir(&repo_dir)
        .env("RUST_LOG", "DEBUG,hyper=info")
        .arg("release-pr")
        .arg("--git-token")
        .arg(&token)
        .arg("--backend")
        .arg("gitea")
        .spawn()
        .unwrap()
        .wait()
        .unwrap();
    assert!(result.success());
    let git_backend = GitBackend::Gitea(
        Gitea::new(
            RepoUrl::new(&repo_url).unwrap(),
            SecretString::from_str(&token).unwrap(),
        )
        .unwrap(),
    );

    let git_client = GitClient::new(git_backend).unwrap();
    let opened_prs = git_client.opened_prs("release-plz/").await.unwrap();
    assert_eq!(opened_prs.len(), 1);
}
