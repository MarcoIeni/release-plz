use anyhow::Context;
use cargo_metadata::Package;
use crates_index::{Crate, GitIndex, SparseIndex};
use tracing::{debug, info};

use std::{
    env,
    io::{BufRead, BufReader},
    path::Path,
    process::{Command, Stdio},
    time::{Duration, Instant},
};

pub enum CargoIndex {
    Git(GitIndex),
    Sparse(SparseIndex),
}

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

pub async fn is_published(index: &mut CargoIndex, package: &Package) -> anyhow::Result<bool> {
    match index {
        CargoIndex::Git(index) => is_published_git(index, package),
        CargoIndex::Sparse(index) => is_in_cache_sparse(index, package).await,
    }
}

pub fn is_published_git(index: &mut GitIndex, package: &Package) -> anyhow::Result<bool> {
    // See if we already have the package in cache.
    if is_in_cache_git(index, package) {
        return Ok(true);
    }

    // The package is not in the cache, so we update the cache.
    index.update()?;

    // Try again with updated index.
    Ok(is_in_cache_git(index, package))
}

fn is_in_cache_git(index: &GitIndex, package: &Package) -> bool {
    let crate_data = index.crate_(&package.name);
    let version = &package.version.to_string();
    is_in_cache(crate_data.as_ref(), version)
}

async fn is_in_cache_sparse(index: &SparseIndex, package: &Package) -> anyhow::Result<bool> {
    let crate_data = fetch_sparse_metadata(index, &package.name).await?;
    let version = &package.version.to_string();
    Ok(is_in_cache(crate_data.as_ref(), version))
}

fn is_in_cache(crate_data: Option<&Crate>, version: &str) -> bool {
    if let Some(crate_data) = crate_data {
        if is_version_present(version, crate_data) {
            return true;
        }
    }
    false
}

fn is_version_present(version: &str, crate_data: &Crate) -> bool {
    crate_data.versions().iter().any(|v| v.version() == version)
}

async fn fetch_sparse_metadata(
    index: &SparseIndex,
    crate_name: &str,
) -> anyhow::Result<Option<Crate>> {
    let req = index.make_cache_request(crate_name)?;
    let (parts, _) = req.body(())?.into_parts();
    let req = http::Request::from_parts(parts, vec![]);

    let req: reqwest::Request = req.try_into()?;

    let client = reqwest::ClientBuilder::new()
        .gzip(true)
        .http2_prior_knowledge()
        .build()?;
    let res = client.execute(req).await?;

    let mut builder = http::Response::builder()
        .status(res.status())
        .version(res.version());

    if let Some(headers) = builder.headers_mut() {
        headers.extend(res.headers().iter().map(|(k, v)| (k.clone(), v.clone())));
    }

    let body = res.bytes().await?;
    let res = builder.body(body.to_vec())?;

    let crate_data = index.parse_cache_response(crate_name, res, true)?;

    Ok(crate_data)
}

pub async fn wait_until_published(index: &mut CargoIndex, package: &Package) -> anyhow::Result<()> {
    let now = Instant::now();
    let sleep_time = Duration::from_secs(2);
    let timeout = Duration::from_secs(300);
    let mut logged = false;

    loop {
        if is_published(index, package).await? {
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

        tokio::time::sleep(sleep_time).await;
    }

    Ok(())
}
