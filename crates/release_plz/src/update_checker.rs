use anyhow::Context;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ClientResponse {
    pub tag_name: String,
}

pub async fn check_update() -> anyhow::Result<()> {
    const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");
    let latest_version = get_latest_version()
        .await
        .context("error while checking for updates")?;
    if latest_version != CURRENT_VERSION {
        println!("A newer version (v{}) is available at https://github.com/MarcoIeni/release-plz - Please Update.", latest_version);
    }
    Ok(())
}

async fn get_latest_version() -> anyhow::Result<String> {
    let client = reqwest::Client::builder()
        .user_agent("release-plz")
        .build()
        .context("can't build GitHub client")?;
    let client_response = client
        .get("https://api.github.com/repos/MarcoIeni/release-plz/releases/latest")
        .send()
        .await
        .context("error while sending request")?;

    let tag_name = client_response
        .json::<ClientResponse>()
        .await
        .ok()
        .context("can't parse response")?
        .tag_name;

    extract_version(&tag_name)
        .with_context(|| {
            format!(
                "can't extract latest release-plz version from tag name {}",
                tag_name
            )
        })
        .map(|v| v.to_string())
}

fn extract_version(tag: &str) -> Option<&str> {
    tag.strip_prefix("release-plz-v")
}

#[cfg(test)]
mod tests {
    use super::extract_version;

    #[test]
    fn version_is_extracted() {
        let tag = "release-plz-v0.2.37";
        assert_eq!(extract_version(tag), Some("0.2.37"));
    }
}
