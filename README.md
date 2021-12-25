# release-plz

[![Crates.io](https://img.shields.io/crates/v/release-plz.svg)](https://crates.io/crates/release-plz)
[![Docs.rs](https://docs.rs/release-plz/badge.svg)](https://docs.rs/release-plz)
[![CI](https://github.com/MarcoIeni/release-plz/workflows/CI/badge.svg)](https://github.com/MarcoIeni/release-plz/actions)

Release-plz will open a PR against your repository and based on Cargo.toml and cargo registry, where it will:
- update your Cargo.toml
- update your Changelog (TODO)

If you run it again, it will also crate tags.

Once you have merged it, you can then use a command like `cargo workspaces publish` to publish the changes.

## Installation

### Requirements

- git

### Cargo

* Install the rust toolchain in order to have cargo installed by following
  [this](https://www.rust-lang.org/tools/install) guide.
* run `cargo install release-plz`

## Flow

1. Start a repository -> release-plz figures out the repository is not published, so it opens a PR where it doesn't change Cargo.toml, but it changes CHANGELOG based on your commits.
2. After you have published 0.1.0, you do other changes -> release-plz opens a PR with both CHANGELOG and Cargo.toml changes.
3. After you just published something, release-plz will not open PRs, because it sees that local project is the same as crates.io.

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
