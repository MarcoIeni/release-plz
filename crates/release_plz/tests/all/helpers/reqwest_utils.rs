#[async_trait::async_trait]
pub trait ReqwestUtils {
    type E;

    async fn ok_if_2xx(self) -> Result<Box<Self>, Self::E>;
}

#[async_trait::async_trait]
impl ReqwestUtils for reqwest::Response {
    type E = anyhow::Error;

    async fn ok_if_2xx(self) -> Result<Box<Self>, Self::E> {
        if self.status().is_success() {
            Ok(Box::new(self))
        } else {
            let response_dbg = format!("{:?}", self);
            let body = self.text().await?;
            anyhow::bail!(
                "Unexpected response. Response: {}. Body: {}",
                response_dbg,
                body
            );
        }
    }
}
