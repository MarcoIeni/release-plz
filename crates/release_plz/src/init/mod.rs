mod gh;

use std::io::Write;

use anyhow::Context;
use cargo_metadata::camino::{Utf8Path, Utf8PathBuf};

const CARGO_REGISTRY_TOKEN: &str = "CARGO_REGISTRY_TOKEN";
const GITHUB_TOKEN: &str = "GITHUB_TOKEN";
const CUSTOM_GITHUB_TOKEN: &str = "RELEASE_PLZ_TOKEN";

pub fn init() -> anyhow::Result<()> {
    ensure_gh_is_installed()?;
    // get the repo url early to verify that the github repository is configured correctly
    let repo_url = gh::repo_url()?;

    greet();
    store_cargo_token()?;

    enable_pr_permissions(&repo_url)?;
    let github_token = store_github_token()?;
    write_actions_yaml(github_token)?;

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
    gh::store_secret(CARGO_REGISTRY_TOKEN)?;
    Ok(())
}

fn enable_pr_permissions(repo_url: &str) -> anyhow::Result<()> {
    println!("
👉 Go to {} and enable the option \"Allow GitHub Actions to create and approve pull requests\". Type Enter when done.", actions_settings_url(repo_url));
    read_stdin()?;
    Ok(())
}

fn store_github_token() -> anyhow::Result<&'static str> {
    let should_create_token = ask_confirmation("👉 Do you want release-plz to use a GitHub Personal Access Token (PAT)? It's required to run CI on release PRs and to run workflows on tags.")?;

    let github_token = if should_create_token {
        println!("
👉 Paste your GitHub PAT.
💡 Create a GitHub PAT following these instructions: https://release-plz.ieni.dev/docs/github/token#use-a-personal-access-token");

        // GitHub custom token
        let release_plz_token: &str = CUSTOM_GITHUB_TOKEN;
        gh::store_secret(release_plz_token)?;
        release_plz_token
    } else {
        // default github token
        GITHUB_TOKEN
    };
    Ok(github_token)
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

fn write_actions_yaml(github_token: &str) -> anyhow::Result<()> {
    let branch = gh::default_branch()?;
    let action_yaml = action_yaml(&branch, github_token);
    fs_err::create_dir_all(actions_file_parent())
        .context("failed to create GitHub actions workflows directory")?;
    fs_err::write(actions_file(), action_yaml).context("error while writing GitHub action file")?;
    Ok(())
}

fn action_yaml(branch: &str, github_token: &str) -> String {
    let checkout_token_line = if github_token == GITHUB_TOKEN {
        format!(
            "
          github_token: ${{{{ secrets.{github_token} }}}}"
        )
    } else {
        "".to_string()
    };
    let with_github_token = if github_token == GITHUB_TOKEN {
        format!(
            "
        with:{checkout_token_line}"
        )
    } else {
        "".to_string()
    };

    format!(
        "name: Release-plz

permissions:
  pull-requests: write
  contents: write

on:
  push:
    branches:
      - {branch}

jobs:
  release-plz-release:
    name: Release-plz release
    runs-on: ubuntu-latest
    steps:
      - name: Checkout repository
        uses: actions/checkout@v4{with_github_token}
      - name: Install Rust toolchain
        uses: dtolnay/rust-toolchain@stable
      - name: Run release-plz
        uses: MarcoIeni/release-plz-action@v0.5
        with:
          command: release
        env:
          GITHUB_TOKEN: ${{{{ secrets.{github_token} }}}}
          CARGO_REGISTRY_TOKEN: ${{{{ secrets.{CARGO_REGISTRY_TOKEN} }}}}

  release-plz-pr:
    name: Release-plz PR
    runs-on: ubuntu-latest
    concurrency:
      group: release-plz-${{{{ github.ref }}}}
      cancel-in-progress: false
    steps:
      - name: Checkout repository
        uses: actions/checkout@v4
        with:
          fetch-depth: 0{checkout_token_line}
      - name: Install Rust toolchain
        uses: dtolnay/rust-toolchain@stable
      - name: Run release-plz
        uses: MarcoIeni/release-plz-action@v0.5
        with:
          command: release-pr
        env:
          GITHUB_TOKEN: ${{{{ secrets.{github_token} }}}}
          CARGO_REGISTRY_TOKEN: ${{{{ secrets.{CARGO_REGISTRY_TOKEN} }}}}
"
    )
}

fn ensure_gh_is_installed() -> anyhow::Result<()> {
    anyhow::ensure!(
        gh::is_gh_installed(),
        "❌ gh cli is not installed. I need it to store GitHub actions repository secrets. Please install it from https://docs.github.com/en/github-cli/github-cli/quickstart");
    Ok(())
}

fn actions_settings_url(repo_url: &str) -> String {
    format!("{}/actions", repo_settings_url(repo_url))
}

fn actions_secret_url(repo_url: &str) -> String {
    format!("{}/secrets/actions", repo_settings_url(repo_url))
}

fn repo_settings_url(repo_url: &str) -> String {
    format!("{repo_url}/settings")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn actions_yaml_string_is_correct() {
        expect_test::expect![[r#"
            name: Release-plz

            permissions:
              pull-requests: write
              contents: write

            on:
              push:
                branches:
                  - main

            jobs:
              release-plz-release:
                name: Release-plz release
                runs-on: ubuntu-latest
                steps:
                  - name: Checkout repository
                    uses: actions/checkout@v4
                    with:
                      github_token: ${{ secrets.GITHUB_TOKEN }}
                  - name: Install Rust toolchain
                    uses: dtolnay/rust-toolchain@stable
                  - name: Run release-plz
                    uses: MarcoIeni/release-plz-action@v0.5
                    with:
                      command: release
                    env:
                      GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
                      CARGO_REGISTRY_TOKEN: ${{ secrets.CARGO_REGISTRY_TOKEN }}

              release-plz-pr:
                name: Release-plz PR
                runs-on: ubuntu-latest
                concurrency:
                  group: release-plz-${{ github.ref }}
                  cancel-in-progress: false
                steps:
                  - name: Checkout repository
                    uses: actions/checkout@v4
                    with:
                      fetch-depth: 0
                  - name: Install Rust toolchain
                    uses: dtolnay/rust-toolchain@stable
                  - name: Run release-plz
                    uses: MarcoIeni/release-plz-action@v0.5
                    with:
                      command: release-pr
                    env:
                      GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
                      CARGO_REGISTRY_TOKEN: ${{ secrets.CARGO_REGISTRY_TOKEN }}
        "#]]
        .assert_eq(&action_yaml("main", GITHUB_TOKEN));
    }

    #[test]
    fn actions_yaml_string_with_custom_token_is_correct() {
        expect_test::expect![[r#"
            name: Release-plz

            permissions:
              pull-requests: write
              contents: write

            on:
              push:
                branches:
                  - main

            jobs:
              release-plz-release:
                name: Release-plz release
                runs-on: ubuntu-latest
                steps:
                  - name: Checkout repository
                    uses: actions/checkout@v4
                  - name: Install Rust toolchain
                    uses: dtolnay/rust-toolchain@stable
                  - name: Run release-plz
                    uses: MarcoIeni/release-plz-action@v0.5
                    with:
                      command: release
                    env:
                      GITHUB_TOKEN: ${{ secrets.RELEASE_PLZ_TOKEN }}
                      CARGO_REGISTRY_TOKEN: ${{ secrets.CARGO_REGISTRY_TOKEN }}

              release-plz-pr:
                name: Release-plz PR
                runs-on: ubuntu-latest
                concurrency:
                  group: release-plz-${{ github.ref }}
                  cancel-in-progress: false
                steps:
                  - name: Checkout repository
                    uses: actions/checkout@v4
                    with:
                      fetch-depth: 0
                  - name: Install Rust toolchain
                    uses: dtolnay/rust-toolchain@stable
                  - name: Run release-plz
                    uses: MarcoIeni/release-plz-action@v0.5
                    with:
                      command: release-pr
                    env:
                      GITHUB_TOKEN: ${{ secrets.RELEASE_PLZ_TOKEN }}
                      CARGO_REGISTRY_TOKEN: ${{ secrets.CARGO_REGISTRY_TOKEN }}
        "#]]
        .assert_eq(&action_yaml("main", CUSTOM_GITHUB_TOKEN));
    }
}
