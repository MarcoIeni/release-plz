use crate::helpers::gitea;
use anyhow::Context;
use chrono::NaiveDate;
use git_cmd::git_in_dir;
use release_plz_core::CARGO_TOML;
use release_plz_core::{
    ChangelogRequest, GitBackend, Gitea, ReleasePrRequest, RepoUrl, UpdateRequest,
};
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::tempdir;

const TEST_REPO: &str = "test_repo";
const USERNAME: &str = "me";

fn copy_gitea(original: &Gitea) -> Gitea {
    Gitea::new(
        RepoUrl::new(gitea::repo_url(&original.remote.owner, &original.remote.repo).as_str())
            .unwrap(),
        original.remote.token.clone(),
    )
    .unwrap()
}

async fn setup(project_dir: &PathBuf, username: String, repo_name: String) -> Gitea {
    let token = gitea::create_user(&username).await;
    gitea::create_repo(&token, &repo_name).await;

    let git_url = gitea::git_cred_url(&username, &repo_name);

    init_repo(project_dir, &git_url);

    let url_repo: String = gitea::repo_url(&username, &repo_name);
    Gitea::new(RepoUrl::new(&url_repo).unwrap(), token.clone().into()).unwrap()
}

fn init_repo(project_dir: &PathBuf, git_url: &str) {
    Command::new("git")
        .arg("clone")
        .arg(git_url)
        .arg(project_dir.as_path().to_str().unwrap())
        .output()
        .unwrap();

    Command::new("cargo")
        .arg("init")
        .arg(project_dir.as_path().to_str().unwrap())
        .output()
        .unwrap();

    git_in_dir(project_dir.as_ref(), &["add", "."]).unwrap();
    git_in_dir(project_dir.as_ref(), &["commit", "-m", "add README"]).unwrap();
}

#[tokio::test]
async fn gitea_client_creates_pr() {
    let expected_pr_title = "chore: release";
    let local_project_dir = tempdir().unwrap();
    let local_project = local_project_dir.as_ref().join(TEST_REPO);

    let user = setup(&local_project, USERNAME.into(), TEST_REPO.into()).await;

    let release_pr_request =
        gitea_release_pr_request(copy_gitea(&user), local_project.as_ref()).unwrap();

    release_plz_core::release_pr(&release_pr_request)
        .await
        .context("could not release PR")
        .unwrap();

    let pulls = gitea::list_pull_requests(&user).await;
    let matching_pull_len = pulls
        .iter()
        .filter(|p| p.title.as_str() == expected_pr_title)
        .count();

    assert_eq!(1, matching_pull_len);
}

fn gitea_release_pr_request(user: Gitea, project: &Path) -> anyhow::Result<ReleasePrRequest> {
    let git = GitBackend::Gitea(user);
    Ok(ReleasePrRequest {
        git,
        update_request: update_request(project),
    })
}

fn update_request(project: &Path) -> UpdateRequest {
    UpdateRequest::new(project.join(CARGO_TOML))
        .unwrap()
        .with_changelog(ChangelogRequest {
            release_date: NaiveDate::from_ymd_opt(2015, 5, 15),
            changelog_config: None,
        })
        .with_registry_project_manifest(project.join(CARGO_TOML))
        .unwrap()
}
