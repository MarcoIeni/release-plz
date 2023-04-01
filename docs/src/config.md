# Configuration

This section describes how you can customize the behavior of release-plz
with the `release-plz.toml` file.

Please note that the configuration file is optional: release-plz is designed to work out of the box,
with decent defaults.

## Reference

The configuration file is written in the [TOML](https://toml.io/) format and consists of the following sections:

- [`[workspace]`](#the-workspace-section) — Defines the default configuration.
  - [`update_dependencies`](#the-update_dependencies-field) — Whether to update dependencies or not.
  - [`changelog_config`](#the-changelog_config-field) — Path to the [git-cliff](https://github.com/orhun/git-cliff) configuration file.
- [`[package]`](#the-package-section) — Defines the package-specific configurations.

### The `[workspace]` section

Defines the global configuration. Applied to all packages by default.
This section is optional, as well as all its fields.

Here's an example configuration:

```toml
[workspace]
update_dependencies = true # update dependencies with `cargo update`
changelog_config = "config/git-cliff.toml"
allow_dirty = true # allow updating repositories with uncommitted changes
# TODO: document these ones, too
# repo_url = "https://github.com/MarcoIeni/release-plz"
# semver_check = "lib"
# update_changelog = true
```

#### The `update_dependencies` field

- If `true`, update all the dependencies in the Cargo.lock file by running `cargo update`.
- If `false`, only update the workspace packages by running `cargo update --workspace`. (default)

#### The `changelog_config` field

Path to the [git-cliff](https://github.com/orhun/git-cliff) configuration file.
If unspecified, release-plz uses the [keep a changelog](https://keepachangelog.com/en/1.1.0/) format.
You can learn more in the [changelog format](changelog-format.md) section.

#### The `allow_dirty` field

- If `true`, allow dirty working directories to be updated.
  The uncommitted changes will be part of the update.
- If `false`, release-plz returns an error if the repository contains uncommitted changes. (default)

Note: This field is different from the `allow-dirty` flag of the `release-plz release` command.
This field only affects the `release-plz update` and `release-plz release-pr` command.

### The `[package]` section

This overrides `workspace`.
Not all settings of `workspace` can be overridden.
