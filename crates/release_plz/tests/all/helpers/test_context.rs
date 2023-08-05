use super::{fake_utils, gitea::GiteaContext};

/// It contains the universe in which release-plz runs.
pub struct TestContext {
    gitea: GiteaContext,
}

impl TestContext {
    pub async fn new() -> Self {
        test_logs::init();
        let repo_name = fake_utils::fake_id();
        Self {
            gitea: GiteaContext::new(repo_name).await,
        }
    }
}
