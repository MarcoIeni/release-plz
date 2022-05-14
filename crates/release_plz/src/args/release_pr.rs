use anyhow::{anyhow, Context};
use git_cmd::Repo;
use git_url_parse::GitUrl;
use release_plz_core::GitHub;
use secrecy::SecretString;

use super::update::Update;

#[derive(clap::Parser, Debug)]
pub struct ReleasePr {
    #[clap(flatten)]
    pub update: Update,
    /// GitHub token used to create the pull request.
    #[clap(long, forbid_empty_values(true))]
    pub github_token: SecretString,
    /// GitHub repository url where your project is hosted.
    /// It defaults to the `origin` url.
    #[clap(long, forbid_empty_values(true))]
    pub repo_url: Option<String>,
}

impl ReleasePr {
    pub fn github(&self) -> anyhow::Result<GitHub> {
        let (owner, repo) = match &self.repo_url {
            Some(url) => owner_and_repo(url),
            None => {
                let project_manifest = self.update.project_manifest();
                let project_dir = project_manifest.parent().context("at least a parent")?;
                let repo = Repo::new(project_dir)?;
                let url = repo.origin_url().context("cannot determine origin url")?;
                owner_and_repo(&url)
            }
        }?;
        Ok(GitHub::new(owner, repo, self.github_token.clone()))
    }
}

fn owner_and_repo(github_url: &str) -> anyhow::Result<(String, String)> {
    let git_url = GitUrl::parse(github_url)
        .map_err(|err| anyhow!("cannot parse github url {}: {}", github_url, err))?;
    let owner = git_url
        .owner
        .with_context(|| format!("cannot find owner in git url {}", github_url))?;
    let repo = git_url.name;
    Ok((owner, repo))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn https_github_url_is_parsed() {
        let expected_owner = "MarcoIeni";
        let expected_repo = "release-plz";
        let url = format!("https://github.com/{}/{}", expected_owner, expected_repo);
        let (owner, repo) = owner_and_repo(&url).unwrap();
        assert_eq!(expected_owner, owner);
        assert_eq!(expected_repo, repo);
    }

    #[test]
    fn git_github_url_is_parsed() {
        let expected_owner = "MarcoIeni";
        let expected_repo = "release-plz";
        let url = format!("git@github.com:{}/{}.git", expected_owner, expected_repo);
        let (owner, repo) = owner_and_repo(&url).unwrap();
        assert_eq!(expected_owner, owner);
        assert_eq!(expected_repo, repo);
    }
}
