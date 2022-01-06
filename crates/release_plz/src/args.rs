use secrecy::SecretString;

#[derive(clap::Parser, Debug)]
#[clap(about, version, author)]
pub struct CliArgs {
    /// GitHub token
    pub github_token: SecretString,
    /// GitHub repository url
    pub repo_url: String,
}
