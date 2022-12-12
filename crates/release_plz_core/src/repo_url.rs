use anyhow::{anyhow, Context};
use git_url_parse::GitUrl;

#[derive(Debug, Clone)]
pub struct RepoUrl {
    pub host: String,
    pub owner: String,
    pub name: String,
}

impl RepoUrl {
    pub fn new(github_url: &str) -> anyhow::Result<Self> {
        let git_url = GitUrl::parse(github_url)
            .map_err(|err| anyhow!("cannot parse github url {}: {}", github_url, err))?;
        let owner = git_url
            .owner
            .with_context(|| format!("cannot find owner in git url {}", github_url))?;
        let name = git_url.name;
        let host = git_url
            .host
            .with_context(|| format!("cannot find host in git url {}", github_url))?;
        Ok(RepoUrl { owner, name, host })
    }

    pub fn is_on_github(&self) -> bool {
        self.host.contains("github")
    }

    /// Get GitHub release link
    pub fn gh_release_link(&self, prev_tag: &str, new_tag: &str) -> String {
        format!(
            "https://{}/{}/{}/compare/{prev_tag}...{new_tag}",
            self.host, self.owner, self.name
        )
    }
}
