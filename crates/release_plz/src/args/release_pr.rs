use std::str::FromStr;

use anyhow::Context;
use clap::builder::NonEmptyStringValueParser;
use clap::ValueEnum;
use git_cmd::Repo;
use git_url_parse::GitUrl;
use release_plz_core::{GitBackend, GitHub, Gitea};
use secrecy::SecretString;
use url::Url;

use super::update::Update;

#[derive(clap::Parser, Debug)]
pub struct ReleasePr {
    #[clap(flatten)]
    pub update: Update,
    /// Git token used to create the pull request.
    #[clap(long, value_parser = NonEmptyStringValueParser::new(), visible_alias = "github_token")]
    token: String,
    /// Kind of git host where your project is hosted.
    #[clap(value_enum, default_value_t = GitBackendKind::Github)]
    backend: GitBackendKind,
}

#[derive(ValueEnum, Clone, Copy, Debug, Eq, PartialEq)]
enum GitBackendKind {
    #[value(name = "github")]
    Github,
    #[value(name = "gitea")]
    Gitea,
}

impl ReleasePr {
    pub fn git_backend(&self) -> anyhow::Result<GitBackend> {
        let url = match &self.repo_url {
            Some(url) => url.clone(),
            None => {
                let project_manifest = self.update.project_manifest();
                let project_dir = project_manifest.parent().context("at least a parent")?;
                let repo = Repo::new(project_dir)?;
                repo.origin_url().context("cannot determine origin url")?
            }
        };

        let parts = git_url_parts(&url)?;
        let token = SecretString::from_str(&self.token).context("Invalid git backend token")?;
        Ok(match self.backend {
            GitBackendKind::Github => {
                GitBackend::Github(GitHub::new(parts.owner, parts.repo, token))
            }
            GitBackendKind::Gitea => GitBackend::Gitea(Gitea::new(
                parts.owner,
                parts.repo,
                token,
                parts.host.context("Gitea backend must have a valid URL")?,
            )?),
        })
    }
}

struct GitUrlParts {
    host: Option<Url>,
    owner: String,
    repo: String,
}

fn git_url_parts(github_url: &str) -> anyhow::Result<GitUrlParts> {
    let git_url = GitUrl::parse(github_url)
        .map_err(|err| anyhow!("cannot parse github url {}: {}", github_url, err))?;
    let host = Url::parse(&format!("{git_url}")).ok();
    let owner = git_url
        .owner
        .with_context(|| format!("cannot find owner in git url {}", github_url))?;
    let repo = git_url.name;
    Ok(GitUrlParts { host, owner, repo })
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
