# Why yet another release tool

New Rust apps and rewrites are mushrooming every day.
Choosing which tools to add to your developer toolbox is becoming harder and harder.

I feel obliged to explain why I created this project and how it compares with similar tools.

## Differences with release-please

Like release-plz, release-please is an open-source project that manages releases via pull requests.
However, there are some differences.

### No configuration needed

Release-please requires two configuration files: `.release-please-manifest.json` and `release-please-config.json`.
Most of the fields you have to write in these two files are already in the `Cargo.toml` files.

Release-plz obtains all the information it needs from the `Cargo.toml` files and the cargo registry,
so no configuration files are required.

### Versions retrieved from the cargo registry

Release-please considers a package "published" only when the relative git tag exists.
Release-please doesn't interact at all with cargo registries.
However, some Rust projects don't create git tags.
Instead, they just publish the package to crates.io.

Release-plz looks at the cargo registry if the relative git tag doesn't exist, making it compatible with both workflows and with the majority of the Rust projects.

### No multiple programming languages support

Release-please supports multiple programming languages, while release-plz only supports Rust projects.

## Differences with other Rust release tools

These are other release tools in the Rust ecosystem, but they primarily focus on the CLI use case, while release-plz focuses mainly on CI.

- [cargo-release](https://github.com/crate-ci/cargo-release):
  - Bumps the version and publishes Rust packages from the CLI.
  - It supports automatic releases with `cargo release --unpublished`
  - It doesn't have first-class support for changelogs. See [this](https://github.com/crate-ci/cargo-release/issues/231) issue.
  - It supports hooks (release-plz doesn't).
- [cargo-workspaces](https://github.com/pksunkara/cargo-workspaces):
  - It's a set of commands to manage cargo workspaces and their crates.
  - The `publish` command releases the packages from the CLI similarly to `cargo-release`.
- [cargo-smart-release](https://github.com/Byron/gitoxide/tree/main/cargo-smart-release):
  - Updates and publishes packages from the CLI.
  - It's very similar to release-plz, but it is not meant to be run in CI (see [this](https://github.com/MarcoIeni/release-plz/issues/13#issuecomment-1065790846) comment from the author).
