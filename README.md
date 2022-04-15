# release-plz

[![Crates.io](https://img.shields.io/crates/v/release-plz.svg)](https://crates.io/crates/release-plz)
[![CI](https://github.com/MarcoIeni/release-plz/workflows/CI/badge.svg)](https://github.com/MarcoIeni/release-plz/actions)
[![Docker](https://badgen.net/badge/icon/docker?icon=docker&label)](https://hub.docker.com/r/marcoieni/release-plz)

Release-plz updates the versions and changelogs of your rust packages, by analyzing you git history,
based on [conventional commits](https://www.conventionalcommits.org/en/v1.0.0/).
Release-plz can update your `Cargo.toml` files locally, or by opening a GitHub Pull Request.

## Publishing

The main goal of the project is to create a fully automated release pipeline.
This means you can easily release changes more frequently, without the fear of doing typo or other
subtle manual mistakes you can make when releasing from your terminal.

Release-plz delegates the process of publishing the packages and creating git tags to other tools.
To publish the packages you updated, you can use one of these commands:

- [cargo release](https://crates.io/crates/cargo-release) (it creates git tags, too):
  ```shell
  cargo release --workspace --execute --no-confirm --token "${TOKEN}"
  ```
- [cargo workspaces](https://crates.io/crates/cargo-workspaces):
  ```shell
  cargo workspaces publish --from-git --token "${TOKEN}"
  ```

## GitHub action

The simplest way to update your project with release-plz is to use the [GitHub action](https://github.com/marketplace/actions/release-plz).

## Installation

### Docker

`docker pull marcoieni/release-plz`

### Cargo

* Install the rust toolchain in order to have cargo installed by following
  [this](https://www.rust-lang.org/tools/install) guide.
* run `cargo install release-plz --locked`

## Example Usage

Make sure you have `git` installed when running `release-plz`.

### Update

With `release-plz update` you can update the version and the changelog of the packages of a local workspace.

In the following example, I run `release-plz` on the `release-plz` project itself.
`Release-plz` increases the version and the changelog of the packages with unpublished changes.

![release-plz update](https://user-images.githubusercontent.com/11428655/160762832-54300ddb-ec9c-4538-a611-c66490c47333.gif)

### Release PR

With `release-plz release-pr` you can open a GitHub Pull Request that updates the version of the packages of a local workspace.

In the following example, I run `release-plz` on the `release-plz` project itself.
`Release-plz` opens a PR that increases the version and the changelog of the packages with unpublished changes.

![release-plz release-pr](https://user-images.githubusercontent.com/11428655/160772903-544c7578-7c17-4311-b6ca-a1aefeabe799.gif)

## Similar projects

- [release-please](https://github.com/googleapis/release-please): release-plz is inspired by release-please,
  but instead of determining the next versions based on git tags, release-plz compares local packages with
  the ones published in the cargo registry.
  Plus, release-plz doesn't need any configuration.
- [cargo smart-release](https://github.com/Byron/gitoxide/tree/main/cargo-smart-release):
  Fearlessly release workspace crates and with beautiful semi-handcrafted changelogs.


## License

Licensed under either of

 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.

See [CONTRIBUTING.md](CONTRIBUTING.md).
