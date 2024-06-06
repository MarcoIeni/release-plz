use std::str::FromStr;

use git_cmd::Repo;
use release_plz_core::{GitClient, GitHub};
use secrecy::SecretString;

#[tokio::main]
async fn main() {
    let token = std::env::var("GITHUB_TOKEN").expect("GITHUB_TOKEN not provided");
    let git_client = GitClient::new(release_plz_core::GitBackend::Github(GitHub::new(
        "MarcoIeni".to_string(),
        "rust-workspace-example".to_string(),
        SecretString::from_str(&token).unwrap(),
    )))
    .unwrap();
    let current_directory = std::env::current_dir().unwrap();
    let current_directory = camino::Utf8Path::from_path(&current_directory).unwrap();
    let repo = Repo::new(current_directory).unwrap();
    let (_current_remote, current_branch) = Repo::get_current_remote_and_branch(current_directory)
        .expect("cannot determine current branch");
    let message = "commit from gh api";
    release_plz_core::git::github_graphql::commit_changes(
        &git_client,
        &repo,
        message,
        &current_branch,
    )
    .await
    .unwrap();
}
