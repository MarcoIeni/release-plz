#[async_trait::async_trait]
pub trait Assert2xx {
    async fn assert_2xx(self) -> Self;
}

#[async_trait::async_trait]
impl Assert2xx for reqwest::Response {
    async fn assert_2xx(self) -> Self {
        if self.status().is_success() {
            self
        } else {
            let response_dbg = format!("{:?}", self);
            let body = self.text().await.unwrap();
            panic!(
                "Unexpected response.
                 Response: {}. Body: {}",
                response_dbg, body
            );
        }
    }
}
