use anyhow::{anyhow, Context};
use git_cmd::Repo;
use git_url_parse::GitUrl;

#[derive(Debug, Clone)]
pub struct RepoUrl {
    pub scheme: String,
    pub host: String,
    port: Option<u16>,
    pub owner: String,
    pub name: String,
}

impl RepoUrl {
    pub fn new(git_host_url: &str) -> anyhow::Result<Self> {
        let git_url = GitUrl::parse(git_host_url)
            .map_err(|err| anyhow!("cannot parse git url {}: {}", git_host_url, err))?;
        let owner = git_url
            .owner
            .with_context(|| format!("cannot find owner in git url {}", git_host_url))?;
        let name = git_url.name;
        let host = git_url
            .host
            .with_context(|| format!("cannot find host in git url {}", git_host_url))?;
        let port = git_url.port;
        let scheme = git_url.scheme.to_string();
        Ok(RepoUrl {
            owner,
            name,
            host,
            port,
            scheme,
        })
    }

    pub fn from_repo(repo: &Repo) -> Result<Self, anyhow::Error> {
        let url = repo.origin_url().context("cannot determine origin url")?;
        RepoUrl::new(&url)
    }

    pub fn is_on_github(&self) -> bool {
        self.host.contains("github")
    }

    /// Get GitHub release link
    pub fn gh_release_link(&self, prev_tag: &str, new_tag: &str) -> String {
        let host = format!("https://{}/{}/{}", self.host, self.owner, self.name);

        if prev_tag == new_tag {
            format!(
                "{host}/releases/tag/{new_tag}"
            )
        } else {
            format!(
                "{host}/compare/{prev_tag}...{new_tag}",
            )
        }
    }

    pub fn gitea_api_url(&self) -> String {
        if let Some(port) = self.port {
            format!("{}://{}:{}/api/v1", self.scheme, self.host, port)
        } else {
            format!("{}://{}/api/v1", self.scheme, self.host)
        }
    }
}
