pub struct CreateUserOption
/// Create a user and return it's token.
fn create_user() -> String {
        let client = reqwest::Client::new();
        client.post(format!("{}/admin/users", base_url()))
            .json(json
}

const fn base_url() -> String {
        "http://localhost:3000/api/v1".to_string()
}
