use anyhow::Context;
use cargo_metadata::{camino::Utf8Path, Package};
use crates_index::{Crate, GitIndex, SparseIndex};
use tracing::{debug, info};

use http::header;
use secrecy::{ExposeSecret, SecretString};
use std::{
    env,
    process::{Command, ExitStatus},
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

pub fn run_cargo(root: &Utf8Path, args: &[&str]) -> anyhow::Result<CmdOutput> {
    debug!("cargo {}", args.join(" "));

    let output = cargo_cmd()
        .current_dir(root)
        .args(args)
        .output()
        .context("cannot run cargo")?;

    let output_stdout = String::from_utf8(output.stdout)?;
    let output_stderr = String::from_utf8(output.stderr)?;

    debug!("cargo stderr: {}", output_stderr);
    debug!("cargo stdout: {}", output_stdout);

    Ok(CmdOutput {
        status: output.status,
        stdout: output_stdout,
        stderr: output_stderr,
    })
}

pub struct CmdOutput {
    pub status: ExitStatus,
    pub stdout: String,
    pub stderr: String,
}

pub async fn is_published(
    index: &mut CargoIndex,
    package: &Package,
    timeout: Duration,
    token: &Option<SecretString>,
) -> anyhow::Result<bool> {
    tokio::time::timeout(timeout, async {
        match index {
            CargoIndex::Git(index) => is_published_git(index, package),
            CargoIndex::Sparse(index) => is_in_cache_sparse(index, package, token).await,
        }
    })
    .await?
    .with_context(|| format!("timeout while publishing {}", package.name))
}

pub fn is_published_git(index: &mut GitIndex, package: &Package) -> anyhow::Result<bool> {
    // See if we already have the package in cache.
    if is_in_cache_git(index, package) {
        return Ok(true);
    }

    // The package is not in the cache, so we update the cache.
    index.update().context("failed to update git index")?;

    // Try again with updated index.
    Ok(is_in_cache_git(index, package))
}

fn is_in_cache_git(index: &GitIndex, package: &Package) -> bool {
    let crate_data = index.crate_(&package.name);
    let version = &package.version.to_string();
    is_in_cache(crate_data.as_ref(), version)
}

async fn is_in_cache_sparse(
    index: &SparseIndex,
    package: &Package,
    token: &Option<SecretString>,
) -> anyhow::Result<bool> {
    let crate_data = fetch_sparse_metadata(index, &package.name, token)
        .await
        .context("failed fetching sparse metadata")?;
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
    token: &Option<SecretString>,
) -> anyhow::Result<Option<Crate>> {
    let req = index.make_cache_request(crate_name)?;
    let (parts, _) = req.body(())?.into_parts();
    let req = http::Request::from_parts(parts, vec![]);

    let mut req: reqwest::Request = req.try_into()?;
    if let Some(token) = token {
        let authorization = token
            .expose_secret()
            .parse()
            .context("parse token as header value")?;
        req.headers_mut()
            .insert(header::AUTHORIZATION, authorization);
    }

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

pub async fn wait_until_published(
    index: &mut CargoIndex,
    package: &Package,
    timeout: Duration,
    token: &Option<SecretString>,
) -> anyhow::Result<()> {
    let now: Instant = Instant::now();
    let sleep_time = Duration::from_secs(2);
    let mut logged = false;

    loop {
        let is_published = is_published(index, package, timeout, token).await?;
        if is_published {
            break;
        } else if timeout < now.elapsed() {
            anyhow::bail!("timeout of {:?} elapsed while publishing the package {}. You can increase this timeout by editing the `publish_timeout` field in the `release-plz.toml` file", timeout, package.name)
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
