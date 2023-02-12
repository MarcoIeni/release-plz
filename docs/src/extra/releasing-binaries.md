# Releasing binaries

## Why release-plz doesn't release binaries

> Since release-plz already publishes GitHub releases, would it
> make sense for it to build the binaries of the project and publish
> them to the release assets? 🤔

Not really. Releasing binaries requires setting a CI job different
from the one used to run `release-plz release` because:
- `release-plz release` should run once (for example on an `ubuntu` CI image);
- building binaries requires a different CI image for each platform
  (e.g. `ubuntu`, `macos`, `windows`).

Since users have to set up an additional CI job to build binaries, using release-plz
would not be more convenient than using a different tool.

Plus, releasing binaries is a complex task, which is already well-handled by other tools
in the Rust ecosystem.

For these reasons, release-plz doesn't build and release binaries.

The next section explains how to use other tools to build and release binaries after
release-plz released the new version of your project.

## Releasing binaries after release

If you are using release-plz to release your project, you can
run a CI job on the "tag" or "release" events to build and release the binaries.

For example,
[here](https://github.com/MarcoIeni/release-plz/blob/main/.github/workflows/cd.yml)
is how release-plz releases binaries for the `release-plz` project itself.

Some projects to consider for this task:
- [upload-rust-binary-action](https://github.com/taiki-e/upload-rust-binary-action):
  GitHub Action for building and uploading Rust binary to GitHub Releases.
- [cargo-dist](https://crates.io/crates/cargo-dist):
  shippable application packaging for Rust.
