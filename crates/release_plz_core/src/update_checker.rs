use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ClientResponse {
    pub tag_name: String,
}

pub async fn get_latest_version() -> String {
    let client = reqwest::Client::builder()
        .user_agent(env!("CARGO_PKG_NAME"))
        .build()
        .unwrap();
    if let Ok(client_response) = client
        .get("https://api.github.com/repos/MarcoIeni/release-plz/releases/latest")
        .send()
        .await
    {
        let tag_name = client_response
            .json::<ClientResponse>()
            .await
            .ok()
            .unwrap()
            .tag_name;
        return tag_name;
    } else {
        "".to_string()
    }
}
