![release-plz-logo](docs/src/assets/robot_text.jpeg)

[![Crates.io](https://img.shields.io/crates/v/release-plz.svg)](https://crates.io/crates/release-plz)
[![CI](https://github.com/MarcoIeni/release-plz/workflows/CI/badge.svg)](https://github.com/MarcoIeni/release-plz/actions)
[![Docker](https://badgen.net/badge/icon/docker?icon=docker&label)](https://hub.docker.com/r/marcoieni/release-plz)

Release-plz updates the versions and changelogs of your rust packages, by analyzing your git history,
based on [conventional commits](https://www.conventionalcommits.org/en/v1.0.0/):
- `release-plz update` updates your project locally, without committing any change.
- `release-plz release-pr` opens a GitHub Pull Request.

Once the changes are merged to the main branch, you can use
`release-plz release` to publish the new versions of the packages.

Here's an example of a release Pull Request created on the release-plz repository itself:

![pr](docs/src/assets/pr.png)

## Docs

Learn how to use release-plz in the mdbook [docs](https://marcoieni.github.io/release-plz/).

## Users

[This](https://github.com/search?type=code&q=path%3A*.yml+OR+path%3A*.yaml+MarcoIeni%2Frelease-plz-action)
search
and [this](https://github.com/MarcoIeni/release-plz-action/network/dependents)
page show the public GitHub repositories using release-plz in CI.

## Similar projects

- [release-please](https://github.com/googleapis/release-please): release-plz is inspired by release-please,
  but instead of determining the next versions based on git tags, release-plz compares local packages with
  the ones published in the cargo registry.
  Plus, release-plz doesn't need any configuration.
- [cargo smart-release](https://github.com/Byron/gitoxide/tree/main/cargo-smart-release):
  Fearlessly release workspace crates and with beautiful semi-handcrafted changelogs.

## Credits

Parts of the codebase are inspired by:
- [cargo-clone](https://github.com/JanLikar/cargo-clone)
- [cargo-edit](https://github.com/killercup/cargo-edit)
- [cargo-release](https://github.com/crate-ci/cargo-release)
- [cargo-workspaces](https://github.com/pksunkara/cargo-workspaces)
- [git-cliff](https://github.com/orhun/git-cliff)

<br>

<sup>
Licensed under either of <a href="LICENSE-APACHE">Apache License, Version 2.0</a>
or <a href="LICENSE-MIT">MIT license</a> at your option.
</sup>

<br>

<sub>
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
</sub>
