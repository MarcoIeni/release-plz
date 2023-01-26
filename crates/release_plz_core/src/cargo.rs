use anyhow::Context;
use cargo_metadata::Package;
use crates_index::Index;
use tracing::{debug, info};

use std::{
    env,
    io::{BufRead, BufReader},
    path::Path,
    process::{Command, Stdio},
    thread::sleep,
    time::{Duration, Instant},
};

fn cargo_cmd() -> Command {
    let cargo = env::var("CARGO").unwrap_or_else(|_| "cargo".to_owned());
    Command::new(cargo)
}

pub fn run_cargo(root: &Path, args: &[&str]) -> anyhow::Result<(String, String)> {
    debug!("cargo {}", args.join(" "));

    let mut stderr_lines = vec![];

    let mut child = cargo_cmd()
        .current_dir(root)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .context("cannot run cargo")?;

    {
        let stderr = child.stderr.as_mut().expect("cannot get child stderr");

        for line in BufReader::new(stderr).lines() {
            let line = line?;

            eprintln!("{line}");
            stderr_lines.push(line);
        }
    }

    let output = child.wait_with_output()?;

    let output_stdout = String::from_utf8(output.stdout)?;
    let output_stderr = stderr_lines.join("\n");

    debug!("cargo stderr: {}", output_stderr);
    debug!("cargo stdout: {}", output_stdout);

    Ok((
        output_stdout.trim().to_owned(),
        output_stderr.trim().to_owned(),
    ))
}

pub fn is_published(index: &mut Index, package: &Package) -> anyhow::Result<bool> {
    // See if we already have the package in cache.
    if is_in_cache(index, package) {
        return Ok(true);
    }

    // The package is not in the cache, so we update the cache.
    index.update()?;

    // Try again with updated index.
    Ok(is_in_cache(index, package))
}

fn is_in_cache(index: &Index, package: &Package) -> bool {
    if let Some(crate_data) = index.crate_(&package.name) {
        if crate_data
            .versions()
            .iter()
            .any(|v| v.version() == package.version.to_string())
        {
            return true;
        }
    }
    false
}

pub fn wait_until_published(index: &mut Index, package: &Package) -> anyhow::Result<()> {
    let now = Instant::now();
    let sleep_time = Duration::from_secs(2);
    let timeout = Duration::from_secs(300);
    let mut logged = false;

    loop {
        if is_published(index, package)? {
            break;
        } else if timeout < now.elapsed() {
            anyhow::bail!("timeout while publishing {}", package.name)
        }

        if !logged {
            info!(
                "waiting for the package {} to be published...",
                package.name
            );
            logged = true;
        }

        sleep(sleep_time);
    }

    Ok(())
}
