![release-plz-logo](https://user-images.githubusercontent.com/11428655/170828599-4ec63822-15dd-4552-b3bc-d32bd6d680f2.jpeg)


[![Crates.io](https://img.shields.io/crates/v/release-plz.svg)](https://crates.io/crates/release-plz)
[![CI](https://github.com/MarcoIeni/release-plz/workflows/CI/badge.svg)](https://github.com/MarcoIeni/release-plz/actions)
[![Docker](https://badgen.net/badge/icon/docker?icon=docker&label)](https://hub.docker.com/r/marcoieni/release-plz)

Release-plz updates the versions and changelogs of your rust packages, by analyzing your git history,
based on [conventional commits](https://www.conventionalcommits.org/en/v1.0.0/):
- `release-plz update` updates your project locally, without committing any change.
- `release-plz release-pr` opens a GitHub Pull Request.

Once the changes are merged to the main branch, you can use
`release-plz release` to publish the new versions of the packages.

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

### Publishing

The goal of release-plz is to create a fully automated release pipeline.
This means you can easily release changes more frequently, without the fear of doing typo or other
subtle manual mistakes you can make when releasing from your terminal.

You can release all the unpublished packages by running `release-plz release`.

## Changelog format

Release-plz generates the changelog by using [git-cliff](https://github.com/orhun/git-cliff).
By default, release-plz uses the [keep a changelog](https://keepachangelog.com/en/1.1.0/) format.
You can customize the changelog format, by providing a git-cliff configuration
file with the `--changelog-config` argument.

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

## Credits

Parts of the codebase are inspired by:
- [cargo-workspaces](https://github.com/pksunkara/cargo-workspaces)
- [cargo-release](https://github.com/crate-ci/cargo-release)
- [cargo-edit](https://github.com/killercup/cargo-edit)
- [git-cliff](https://github.com/orhun/git-cliff)
