use std::str::FromStr;

use anyhow::Context;
use clap::builder::NonEmptyStringValueParser;
use clap::ValueEnum;
use release_plz_core::{GitBackend, GitHub, GitLab, Gitea};
use secrecy::SecretString;

use super::update::Update;

#[derive(clap::Parser, Debug)]
pub struct ReleasePr {
    #[command(flatten)]
    pub update: Update,
    /// Git token used to create the pull request.
    #[arg(long, value_parser = NonEmptyStringValueParser::new(), visible_alias = "github-token")]
    git_token: String,
    /// Kind of git host where your project is hosted.
    #[arg(long, value_enum, default_value_t = GitBackendKind::Github)]
    backend: GitBackendKind,
}

#[derive(ValueEnum, Clone, Copy, Debug, Eq, PartialEq)]
pub enum GitBackendKind {
    #[value(name = "github")]
    Github,
    #[value(name = "gitea")]
    Gitea,
    #[value(name = "gitlab")]
    Gitlab,
}

impl ReleasePr {
    pub fn git_backend(&self) -> anyhow::Result<GitBackend> {
        let repo = self.update.repo_url()?;

        let token = SecretString::from_str(&self.git_token).context("Invalid git backend token")?;
        Ok(match self.backend {
            GitBackendKind::Github => {
                anyhow::ensure!(
                    repo.is_on_github(),
                    "Can't create PR: the repository is not hosted in GitHub. Please select a different backend."
                );
                GitBackend::Github(GitHub::new(repo.owner, repo.name, token))
            }
            GitBackendKind::Gitea => GitBackend::Gitea(Gitea::new(repo, token)?),
            GitBackendKind::Gitlab => GitBackend::Gitlab(GitLab::new(repo.owner, repo.name, token)),
        })
    }
}

#[cfg(test)]
mod tests {
    use release_plz_core::RepoUrl;

    const GITHUB_COM: &str = "github.com";

    #[test]
    fn https_github_url_is_parsed() {
        let expected_owner = "MarcoIeni";
        let expected_repo = "release-plz";
        let url = format!("https://{GITHUB_COM}/{expected_owner}/{expected_repo}");
        let repo = RepoUrl::new(&url).unwrap();
        assert_eq!(expected_owner, repo.owner);
        assert_eq!(expected_repo, repo.name);
        assert_eq!(GITHUB_COM, repo.host);
        assert!(repo.is_on_github())
    }

    #[test]
    fn git_github_url_is_parsed() {
        let expected_owner = "MarcoIeni";
        let expected_repo = "release-plz";
        let url = format!("git@github.com:{expected_owner}/{expected_repo}.git");
        let repo = RepoUrl::new(&url).unwrap();
        assert_eq!(expected_owner, repo.owner);
        assert_eq!(expected_repo, repo.name);
        assert_eq!(GITHUB_COM, repo.host);
        assert!(repo.is_on_github())
    }

    #[test]
    fn gitea_url_is_parsed() {
        let host = "example.com";
        let expected_owner = "MarcoIeni";
        let expected_repo = "release-plz";
        let url = format!("https://{host}/{expected_owner}/{expected_repo}");
        let repo = RepoUrl::new(&url).unwrap();
        assert_eq!(expected_owner, repo.owner);
        assert_eq!(expected_repo, repo.name);
        assert_eq!(host, repo.host);
        assert_eq!("https", repo.scheme);
        assert!(!repo.is_on_github());
        assert_eq!(format!("https://{host}/api/v1/"), repo.gitea_api_url());
    }
}
