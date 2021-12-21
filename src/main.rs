use std::{
    collections::HashMap,
    env,
    path::{Path, PathBuf},
    process::Command,
};

use semver::Version;

struct LocalPackage {
    version: Version,
    next_version: Option<Version>,
    hash: String,
}

fn calculate_local_crates(crates: impl Iterator<Item = PathBuf>) -> HashMap<PathBuf, LocalPackage> {
    for c in crates {
        let hash = hash_dir(c);
    }
    todo!()
}

fn main() -> anyhow::Result<()> {
    install_dependencies()?;
    // TODO download in tmp directory
    download_crate("rust-gh-example")?;
    let local_crates = list_crates(&PathBuf::from("."))?;
    let remote_crates = list_crates(&PathBuf::from("rust-gh-example"))?;
    println!("crates: {:?}", remote_crates);

    // pr command:
    // - go back commit by commit and for every local crate:
    //   - If the local crate was edited in that commit:
    //     - if the hash of that crate is the same of the remote crate, that local crate is done.
    //     - otherwise:
    //       - add the entry to the changelog of that crate.
    //       - bump the version of that crate according to the semantic versioning of the commit.
    // - raise PR

    // release command (probably this is already done in ):
    // - for every local_crate with a version != remote one:
    //   - publish crate
    //   - create a new tag with format `local_crate v*new_version*`
    // // Maybe the same or similar is done by :
    // // cargo workspaces publish  --from-git --token "${TOKEN}" --yes
    Ok(())
}

fn install_dependencies() -> anyhow::Result<()> {
    for program in ["cargo-workspaces", "cargo-clone", "sha1dir"] {
        Command::new("cargo").args(["install", program]).output()?;
    }
    Ok(())
}

fn list_crates(directory: &Path) -> anyhow::Result<Vec<PathBuf>> {
    let prev_dir = env::current_dir()?;
    env::set_current_dir(directory)?;
    let output = Command::new("cargo")
        .args(["workspaces", "list", "--long"])
        .output();
    env::set_current_dir(prev_dir)?;
    let output = output?.stdout;

    let output = String::from_utf8(output)?;
    let paths = output.lines().map(|l| {
        l.rsplit(' ')
            .next()
            .expect("no new line in cargo workspaces output")
    });

    Ok(paths.map(PathBuf::from).collect())
}

fn download_crate(crate_name: &str) -> anyhow::Result<()> {
    Command::new("cargo").args(["clone", crate_name]).output()?;
    Ok(())
}

fn hash_dir(dir: impl AsRef<Path>) -> anyhow::Result<String> {
    let output = Command::new("sha1dir").arg(dir.as_ref()).output()?;
    let output = String::from_utf8(output.stdout)?;
    let sha1 = output
        .split(' ')
        .into_iter()
        .next()
        .expect("cannot calculate hash");

    Ok(sha1.to_string())
}
