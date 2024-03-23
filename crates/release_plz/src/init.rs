use std::{io::Write, process::Command};

use anyhow::Context;
use cargo_metadata::camino::{Utf8Path, Utf8PathBuf};

pub fn init() -> anyhow::Result<()> {
    ensure_gh_is_installed()?;
    // get the repo url early to verify that the github repository is configured correctly
    let repo_url = repo_url()?;

    greet();
    store_cargo_token()?;

    enable_pr_permissions(&repo_url)?;
    store_github_token()?;
    write_actions_yaml()?;

    print_recap(&repo_url);
    Ok(())
}

fn actions_file_parent() -> Utf8PathBuf {
    Utf8Path::new(".github").join("workflows")
}

fn actions_file() -> Utf8PathBuf {
    actions_file_parent().join("release-plz.yml")
}

fn greet() {
    println!("👋 This process will guide you in setting up release-plz in your GitHub repository, using `gh` (the GitHub CLI) to store the necessary tokens in your repository secrets.");
}

fn store_cargo_token() -> anyhow::Result<()> {
    println!("👉 Paste your cargo registry token to store it in the GitHub actions repository secrets.
💡 You can create a crates.io token on https://crates.io/settings/tokens/new, specifying the following scopes: \"publish-new\" and \"publish-update\".");
    store_secret("CARGO_REGISTRY_TOKEN")?;
    Ok(())
}

fn enable_pr_permissions(repo_url: &str) -> anyhow::Result<()> {
    println!("
👉 Go to {} and enable the option \"Allow GitHub Actions to create and approve pull requests\". Type Enter when done.", actions_settings_url(repo_url));
    read_stdin()?;
    Ok(())
}

fn store_github_token() -> anyhow::Result<()> {
    let should_create_token = ask_confirmation("👉 Do you want release-plz to use a GitHub Personal Access Token (PAT)? It's required to run CI on release PRs and to run workflows on tags.")?;

    if should_create_token {
        println!("
👉 Paste your GitHub PAT.
💡 Create a GitHub PAT following these instructions: https://release-plz.ieni.dev/docs/github/token#use-a-personal-access-token");
        store_secret("RELEASE_PLZ_TOKEN")?;
    }
    Ok(())
}

fn print_recap(repo_url: &str) {
    println!(
        "All done 🎉
- GitHub action file written to {}
- GitHub action secrets stored. Review them at {}

Enjoy automated releases 🤖",
        actions_file(),
        actions_secret_url(repo_url)
    );
}

fn read_stdin() -> anyhow::Result<String> {
    let mut input = String::new();
    std::io::stdin()
        .read_line(&mut input)
        .context("error while reading user input")?;
    Ok(input)
}

fn ask_confirmation(question: &str) -> anyhow::Result<bool> {
    print!("{question} (Y/n) ");
    std::io::stdout().flush().unwrap();
    let input = read_stdin()?;
    let input = input.trim().to_lowercase();
    Ok(input != "n")
}

fn write_actions_yaml() -> anyhow::Result<()> {
    let branch = default_branch()?;
    let action_yaml = action_yaml(&branch);
    fs_err::create_dir_all(actions_file_parent())
        .context("failed to create GitHub actions workflows directory")?;
    fs_err::write(actions_file(), action_yaml).context("error while writing GitHub action file")?;
    Ok(())
}

fn action_yaml(branch: &str) -> String {
    let head = r#"name: Release Plz

permissions:
  pull-requests: write
  contents: write

on:
  push:
    branches:
      - "#;
    let jobs = r#"

jobs:
  release-plz:
    name: Release-plz
    runs-on: ubuntu-latest
    steps:
      - name: Checkout repository
        uses: actions/checkout@v4
        with:
          fetch-depth: 0
      - name: Install Rust toolchain
        uses: dtolnay/rust-toolchain@stable
      - name: Run release-plz
        uses: MarcoIeni/release-plz-action@v0.5
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
          CARGO_REGISTRY_TOKEN: ${{ secrets.CARGO_REGISTRY_TOKEN }}
"#;
    format!("{head}{branch}{jobs}")
}

/// Store secret reading it from stdin.
fn store_secret(token_name: &str) -> anyhow::Result<()> {
    let output = std::process::Command::new("gh")
        .arg("secret")
        .arg("set")
        .arg(token_name)
        .spawn()
        .context("error while spawning gh to set repository secret")?
        .wait_with_output()
        .context("error while waiting gh to set repository secret")?;
    get_stdout_if_success(output).context("error while setting repository secret")?;
    println!();
    Ok(())
}

fn ensure_gh_is_installed() -> anyhow::Result<()> {
    anyhow::ensure!(
        is_gh_installed(),
        "❌ gh cli is not installed. I need it to store GitHub actions repository secrets. Please install it from https://docs.github.com/en/github-cli/github-cli/quickstart");
    Ok(())
}

fn is_gh_installed() -> bool {
    Command::new("gh")
        .arg("version")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

fn default_branch() -> anyhow::Result<String> {
    let output = Command::new("gh")
        .arg("repo")
        .arg("view")
        .arg("--json")
        .arg("defaultBranchRef")
        .arg("--jq")
        .arg(".defaultBranchRef.name")
        .output()
        .with_context(|| "error while running gh to retrieve current branch")?;
    let branch = get_stdout_if_success(output)
        .context("error while running gh to retrieve current branch")?;
    Ok(branch)
}

fn get_stdout_if_success(output: std::process::Output) -> anyhow::Result<String> {
    if !output.status.success() {
        let stderr = String::from_utf8(output.stderr).unwrap_or_default();
        anyhow::bail!(stderr);
    }
    let stdout = String::from_utf8(output.stdout)
        .context("can't read stdout")?
        .trim()
        .to_string();
    Ok(stdout)
}

fn repo_url() -> anyhow::Result<String> {
    let output = Command::new("gh")
        .arg("repo")
        .arg("view")
        .arg("--json")
        .arg("url")
        .arg("-q")
        .arg(".url")
        .output()
        .with_context(|| "error while running gh to retrieve current repository")?;
    let url = get_stdout_if_success(output)
        .context("error while running gh to retrieve current branch")?;
    Ok(url)
}

fn actions_settings_url(repo_url: &str) -> String {
    format!("{}/actions", repo_settings_url(repo_url))
}

fn actions_secret_url(repo_url: &str) -> String {
    format!("{}/secrets/actions", repo_settings_url(repo_url))
}

fn repo_settings_url(repo_url: &str) -> String {
    format!("{}/settings", repo_url)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn actions_yaml_string_is_correct() {
        expect_test::expect![[r#"
            name: Release Plz

            permissions:
              pull-requests: write
              contents: write

            on:
              push:
                branches:
                  - main

            jobs:
              release-plz:
                name: Release-plz
                runs-on: ubuntu-latest
                steps:
                  - name: Checkout repository
                    uses: actions/checkout@v4
                    with:
                      fetch-depth: 0
                  - name: Install Rust toolchain
                    uses: dtolnay/rust-toolchain@stable
                  - name: Run release-plz
                    uses: MarcoIeni/release-plz-action@v0.5
                    env:
                      GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
                      CARGO_REGISTRY_TOKEN: ${{ secrets.CARGO_REGISTRY_TOKEN }}
        "#]]
        .assert_eq(&action_yaml("main"));
    }
}
