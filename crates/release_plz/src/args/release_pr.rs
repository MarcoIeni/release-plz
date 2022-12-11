use std::str::FromStr;

use anyhow::Context;
use clap::builder::NonEmptyStringValueParser;
use release_plz_core::GitHub;
use secrecy::SecretString;

use super::update::Update;

#[derive(clap::Parser, Debug)]
pub struct ReleasePr {
    #[clap(flatten)]
    pub update: Update,
    /// GitHub token used to create the pull request.
    #[clap(long, value_parser = NonEmptyStringValueParser::new())]
    github_token: String,
}

impl ReleasePr {
    pub fn github(&self) -> anyhow::Result<GitHub> {
        let repo = self.update.repo_url()?;
        anyhow::ensure!(
            repo.is_on_github(),
            "Can't create PR: the repository is not hosted in GitHub"
        );
        let token = SecretString::from_str(&self.github_token).context("Invalid GitHub token")?;
        Ok(GitHub::new(repo.owner, repo.name, token))
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
        let url = format!("https://{GITHUB_COM}/{}/{}", expected_owner, expected_repo);
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
        let url = format!("git@github.com:{}/{}.git", expected_owner, expected_repo);
        let repo = RepoUrl::new(&url).unwrap();
        assert_eq!(expected_owner, repo.owner);
        assert_eq!(expected_repo, repo.name);
        assert_eq!(GITHUB_COM, repo.host);
        assert!(repo.is_on_github())
    }
}
