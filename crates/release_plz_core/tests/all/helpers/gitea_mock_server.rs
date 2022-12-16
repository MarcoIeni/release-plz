use url::Url;
use wiremock::{
    matchers::{method, path},
    Mock, MockServer, ResponseTemplate,
};

pub struct GiteaMockServer {
    server: MockServer,
    owner: &'static str,
    repo: &'static str,
}

impl GiteaMockServer {
    pub async fn start(owner: &'static str, repo: &'static str) -> Self {
        Self {
            server: MockServer::start().await,
            owner,
            repo,
        }
    }

    pub fn base_url(&self) -> Url {
        Url::parse(&self.server.uri()).unwrap()
    }

    /// Nobody tried to open a PR.
    pub async fn expect_no_created_prs(&self) {
        Mock::given(method("POST"))
            .and(path(self.pulls_path()))
            .respond_with(ResponseTemplate::new(200))
            .expect(0)
            .mount(&self.server)
            .await;
    }

    /// Return an empty list of PRs.
    pub async fn no_prs(&self) {
        let no_prs = {
            let empty_body = serde_json::Value::Array(vec![]);
            ResponseTemplate::new(200).set_body_json(empty_body)
        };
        Mock::given(method("GET"))
            .and(path(self.pulls_path()))
            .respond_with(no_prs)
            .expect(1)
            .mount(&self.server)
            .await;
    }

    fn pulls_path(&self) -> String {
        format!("/api/v1/repos/{}/{}/pulls", self.owner, self.repo)
    }
}
