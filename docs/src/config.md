# Configuration

This section describes how you can customize the behavior of release-plz
with the `release-plz.toml` file.

This configuration file is optional — release-plz is designed to work out of the box,
with decent defaults.

## Reference

The configuration file is written in the [TOML](https://toml.io/) format and consists of
the following sections:

- [`[workspace]`](#the-workspace-section) — Defines the default configuration.
  - [`update_dependencies`](#the-update_dependencies-field) — Update all dependencies.
  - [`changelog_config`](#the-changelog_config-field) — Path to the [git-cliff] configuration file.
  - [`allow_dirty`](#the-allow_dirty-field) — Update dirty working directories.
  - [`repo_url`](#the-repo_url-field) — Repository URL.
  - [`semver_check`](#the-semver_check-field) — Run [cargo-semver-checks].
  - [`update_changelog`](#the-update_changelog-field) — Update changelog.
- [`[package]`](#the-package-section) — Defines the package-specific configurations.
  - [`semver_check`](#the-semver_check-field-package-section) — Run [cargo-semver-checks].
  - [`update_changelog`](#the-update_changelog-field-package-section) — Update changelog.

### The `[workspace]` section

Defines the global configuration, applied to all packages by default.
This section is optional, as well as all its fields.

Here's an example configuration:

```toml
[workspace]
update_dependencies = true # update dependencies with `cargo update`
changelog_config = "config/git-cliff.toml"
allow_dirty = true # allow updating repositories with uncommitted changes
repo_url = "https://github.com/<owner>/<repo>"
semver_check = "no"
update_changelog = false
```

#### The `update_dependencies` field

- If `true`, update all the dependencies in the `Cargo.lock` file by running `cargo update`.
- If `false`, only update the workspace packages by running `cargo update --workspace`. *(Default)*.

#### The `changelog_config` field

Path to the [git-cliff] configuration file.
If unspecified, release-plz uses the [keep a changelog](https://keepachangelog.com/en/1.1.0/) format.
You can learn more in the [changelog format](changelog-format.md) section.

#### The `allow_dirty` field

- If `true`, allow release-plz to update dirty working directories.
  A directory is considered dirty if it contains uncommitted changes.
  The uncommitted changes will be part of the update.
- If `false`, release-plz returns an error if the repository contains uncommitted changes. *(Default)*.

Note: This field is different from the `allow-dirty` flag of the `release-plz release` command.
This field only affects the `release-plz update` and `release-plz release-pr` command.

#### The `repo_url` field

GitHub/Gitea repository URL where your project is hosted.
It is used to generate the changelog release link and open the PR.
Normally, you don't need to set this field,
because release-plz defaults to the URL of the default git remote.

#### The `semver_check` field

With this field, you can tell release-plz when to run [cargo-semver-checks]:

- If `"no"`, never run it.
- If `"yes"`, always run it.
- If `"lib"`, run it if the package is a library. *(Default)*.

This field can be overridden in the [`[package]`](#the-package-section) section.

#### The `update_changelog` field

- If `true`, update the changelog of the crates. *(Default)*.
- If `false`, don't update changelogs.

This field can be overridden in the [`[package]`](#the-package-section) section.

### The `[package]` section

In this section, you can override some of the `workspace` fields for specific packages.
This section is optional, as well as all its fields.

```toml
[package.my_package]
semver_check = "no"
update_changelog = false
```

#### The `semver_check` field (`package` section)

- If `"yes"`, run [cargo-semver-checks] for this package.
- If `"no"`, don't.

By default, release-plz runs [cargo-semver-checks] if the package is a library.

#### The `update_changelog` field (`package` section)

- If `true`, update the changelog of this package. *(Default)*.
- If `false`, don't.

[cargo-semver-checks]: https://github.com/obi1kenobi/cargo-semver-checks
[git-cliff]: https://github.com/orhun/git-cliff
