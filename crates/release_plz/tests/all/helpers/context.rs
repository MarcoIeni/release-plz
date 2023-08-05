use super::gitea::GiteaContext;

/// Test context. It contains the universe in which release-plz runs.
pub struct Context {
    gitea: GiteaContext,
}

impl Context {
    pub async fn new() -> Self {
        test_logs::init();
        Self {
            gitea: GiteaContext::new("myrepo".to_string()).await,
        }
    }
}
