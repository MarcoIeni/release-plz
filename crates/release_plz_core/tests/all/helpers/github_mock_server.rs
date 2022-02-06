use url::Url;
use wiremock::{matchers::any, Mock, MockServer, ResponseTemplate};

pub struct GitHubMockServer {
    server: MockServer,
}

impl GitHubMockServer {
    pub async fn start() -> Self {
        Self {
            server: MockServer::start().await,
        }
    }

    pub fn base_url(&self) -> Url {
        Url::parse(&self.server.uri()).unwrap()
    }

    /// Api was never called
    pub async fn radio_silence(&self) {
        Mock::given(any())
            .respond_with(ResponseTemplate::new(200))
            .expect(0)
            .mount(&self.server)
            .await;
    }
}
