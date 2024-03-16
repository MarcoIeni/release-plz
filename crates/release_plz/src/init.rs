use std::process::Command;

use anyhow::Context;

const ACTIONS_FILE: &str = ".github/workflows/release-plz.yml";

pub fn init() -> anyhow::Result<()> {
    anyhow::ensure!(
        is_gh_installed(),
        "gh cli is not installed. I need it to store GitHub actions repository secrets. Please install it from https://docs.github.com/en/github-cli/github-cli/quickstart");
    println!("Paste your cargo registry token to store it in the GitHub actions repository secrets.
Create a crates.io token on https://crates.io/settings/tokens/new, specifying the following scopes: \"publish-new\" and \"publish-update\".");
    store_secret("CARGO_REGISTRY_TOKEN")?;

    write_actions_yaml()?;
    println!("GitHub action file written to {ACTIONS_FILE}");

    Ok(())
}

fn write_actions_yaml() -> anyhow::Result<()> {
    let action_yaml = action_yaml();
    fs_err::create_dir_all(ACTIONS_FILE).context("failed to create actions yaml file")?;
    fs_err::write(ACTIONS_FILE, action_yaml).context("error while writing GitHub action file")?;
    Ok(())
}

fn action_yaml() -> &'static str {
    r#"name: Release Plz

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
"#
}

/// Store secret reading it from stdin.
fn store_secret(token_name: &str) -> anyhow::Result<()> {
    let mut command = std::process::Command::new("gh");
    command.arg("secret").arg("set").arg(token_name);
    let output = command
        .spawn()
        .context("error while spawning gh to set repository secret")?
        .wait_with_output()
        .context("error while waiting gh to set repository secret")?;
    anyhow::ensure!(
        output.status.success(),
        "error while setting repository secret"
    );
    Ok(())
}

pub fn is_gh_installed() -> bool {
    Command::new("gh")
        .arg("version")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}
