use std::process::Output;

use anyhow::Context;
use tracing::instrument;

#[instrument(skip_all)]
pub fn stdout(output: Output) -> anyhow::Result<String> {
    string_from_bytes(output.stdout)
}

#[instrument(skip_all)]
pub fn stderr(output: Output) -> anyhow::Result<String> {
    string_from_bytes(output.stderr)
}

fn string_from_bytes(bytes: Vec<u8>) -> anyhow::Result<String> {
    let stdout = String::from_utf8(bytes).context("cannot extract stderr")?;
    let stdout = stdout.trim();
    Ok(stdout.to_string())
}
