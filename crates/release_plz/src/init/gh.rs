use std::process::Command;

use anyhow::Context;

pub fn repo_url() -> anyhow::Result<String> {
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

/// Store secret reading it from stdin.
pub fn store_secret(token_name: &str) -> anyhow::Result<()> {
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

pub fn is_gh_installed() -> bool {
    Command::new("gh")
        .arg("version")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

pub fn default_branch() -> anyhow::Result<String> {
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
