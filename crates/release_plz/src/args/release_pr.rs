use super::{update::Update, OutputType};

#[derive(clap::Parser, Debug)]
pub struct ReleasePr {
    #[command(flatten)]
    pub update: Update,
    /// Output format. If specified, prints the branch, URL and number of
    /// the release PR, if any.
    #[arg(short, long, value_enum)]
    pub output: Option<OutputType>,
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
        assert!(repo.is_on_github());
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
        assert!(repo.is_on_github());
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
