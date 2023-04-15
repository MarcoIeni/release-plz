# Configuration

This section describes how you can customize the behavior of release-plz
with the `release-plz.toml` file.

This configuration file is optional — release-plz is designed to work out of the box,
with decent defaults.

Put the `release-plz.toml` file in the same directory of your root `Cargo.toml`.

## Reference

The configuration file is written in the [TOML](https://toml.io/) format and consists of
the following sections:

- [`[workspace]`](#the-workspace-section) — Default configuration.
  - [`update_dependencies`](#the-update_dependencies-field) — Update all dependencies.
  - [`changelog_config`](#the-changelog_config-field) — Path to the [git-cliff] configuration file.
  - [`allow_dirty`](#the-allow_dirty-field) — Update dirty working directories.
  - [`repo_url`](#the-repo_url-field) — Repository URL.
  - [`semver_check`](#the-semver_check-field) — Run [cargo-semver-checks].
  - [`update_changelog`](#the-update_changelog-field) — Update changelog.
  - [`git_release_enable`](#the-git_release_enable-field-package-section) — Enable git release.
  - [`publish_allow_dirty`](#the-publish_allow_dirty-field) — Package dirty directories.
  - [`publish_no_verify`](#the-publish_no_verify-field) — Don't verify package build.
- [`[[package]]`](#the-package-section) — Package-specific configurations.
  - [`name`](#the-name-field) — Package name.
  - [`semver_check`](#the-semver_check-field-package-section) — Run [cargo-semver-checks].
  - [`update_changelog`](#the-update_changelog-field-package-section) — Update changelog.
  - [`changelog_path`](#the-changelog_path-field-package-section) — Changelog path.
  - [`git_release_enable`](#the-git_release_enable-field-package-section) — Enable git release.
  - [`publish_allow_dirty`](#the-publish_allow_dirty-field-package-section) — Package dirty directories.
  - [`publish_no_verify`](#the-publish_no_verify-field-package-section) —
    Don't verify package build.

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
semver_check = false
update_changelog = false
git_release_enable = true
publish_allow_dirty = true
publish_no_verify = false
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

- If `false`, never run it.
- If `true`, always run it.
- If unspecified, run it if the package is a library. *(Default)*.

This field can be overridden in the [`[package]`](#the-package-section) section.

#### The `update_changelog` field

- If `true`, update the changelog of the crates. *(Default)*.
- If `false`, don't update changelogs.

This field can be overridden in the [`[package]`](#the-package-section) section.

#### The `git_release_enable` field

- If `true`, release-plz will create a git release for the created tag. *(Default)*.
- If `false`, release-plz will not create a git release.

The supported git releases are:

- [GitHub](https://docs.github.com/en/repositories/releasing-projects-on-github/managing-releases-in-a-repository)
- [Gitea](https://docs.gitea.io/en-us/)
- [GitLab](https://docs.gitlab.com/ee/user/project/releases/#releases)

#### The `publish_allow_dirty` field

Allow dirty working directories to be packaged.
When `true`, `release-plz` adds the `--allow-dirty` flag to `cargo publish`.

#### The `publish_no_verify` field

Don't verify the contents by building them.
When `true`, `release-plz` adds the `--no-verify` flag to `cargo publish`.

### The `[[package]]` section

In this section, you can override some of the `workspace` fields for specific packages.
This section is optional, as well as all its fields.

Here's an example configuration:

```toml
[[package]]
name = "my_package"
semver_check = false
update_changelog = false
changelog_path = "docs/CHANGELOG.md"
```

#### The `name` field

Name of the package to which the configuration applies.
*(Mandatory field)*.

#### The `semver_check` field (`package` section)

- If `true`, run [cargo-semver-checks] for this package.
- If `false`, don't.

By default, release-plz runs [cargo-semver-checks] if the package is a library.

#### The `update_changelog` field (`package` section)

- If `true`, update the changelog of this package. *(Default)*.
- If `false`, don't.

#### The `changelog_path` field (`package` section)

By default, release-plz looks for the changelog in the `CHANGELOG.md` file
of the same directory of the `Cargo.toml` of the package:

```txt
.
├── src/
├── CHANGELOG.md
└── Cargo.toml
```

If the changelog of a package is in a different location, you can specify it with this field.

This field is relative to the root `Cargo.toml` file.
In GitHub actions, this is the root of the repository.

This field cannot be set in the `[workspace]` section.

[cargo-semver-checks]: https://github.com/obi1kenobi/cargo-semver-checks
[git-cliff]: https://github.com/orhun/git-cliff

#### The `git_release_enable` field (`package` section)

Overrides the [`workspace.git_release_enable`](#the-git_release_enable-field) field.

#### The `publish_allow_dirty` field (`package` section)

Overrides the
[`workspace.publish_allow_dirty`](#the-publish_allow_dirty-field) field.

#### The `publish_no_verify` field (`package` section)

Overrides the [`workspace.publish_no_verify`](#the-publish_no_verify-field) field.
