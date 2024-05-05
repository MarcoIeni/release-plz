use anyhow::Context;
use reqwest::Response;

#[async_trait::async_trait]
pub trait ReqwestUtils {
    async fn ok_if_2xx(self) -> anyhow::Result<Response>;
}

#[async_trait::async_trait]
impl ReqwestUtils for Response {
    async fn ok_if_2xx(self) -> anyhow::Result<Self> {
        if self.status().is_success() {
            Ok(self)
        } else {
            let response_dbg = format!("{:?}", &self);
            let body = self.text().await.context("can't convert body to text")?;
            anyhow::bail!(
                "Unexpected response. \
                 Response: {response_dbg}. \
                 Body: {body}",
            );
        }
    }
}
