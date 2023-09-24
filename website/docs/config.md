# Configuration

This section describes how you can customize the behavior of release-plz
with the `release-plz.toml` file.

This configuration file is optional — release-plz is designed to work out of the box,
with decent defaults.

Put the `release-plz.toml` file in the same directory of your root `Cargo.toml`.

## Example

Here's an example configuration file for a cargo workspace.
`package_a` and `package_b` override some fields from the default configuration `[workspace]`,
while the other packages inherit the default configuration.

```toml
[workspace]
allow_dirty = true # allow updating repositories with uncommitted changes
changelog_config = "config/git-cliff.toml" # use a custom git-cliff configuration
changelog_update = false # disable changelog updates
dependencies_update = true # update dependencies with `cargo update`
git_release_enable = false # disable GitHub/Gitea releases
pr_labels = ["release"] # add the `release` label to the release Pull Request
publish_allow_dirty = true # add `--allow-dirty` to `cargo publish`
semver_check = false # disable API breaking changes checks

[[package]] # the double square brackets define a TOML table array
name = "package_a"
changelog_include = ["package_b"] # include commits from `package_b` in the changelog
changelog_path = "docs/CHANGELOG.md" # use a custom changelog path for `package_a`
changelog_update = true # enable changelog update for `package_a`
git_release_enable = true # enable GitHub/Gitea releases for `package_a`
publish = false # disable `cargo publish` for `package_a`

[[package]]
name = "package_b"
semver_check = true # enable semver_check for `package_b`
publish_no_verify = true # add `--no-verify` to `cargo publish` for `package_b`
```

## Reference

The configuration file is written in the [TOML](https://toml.io/) format and consists of
the following sections:

- [`[workspace]`](#the-workspace-section) — Default configuration.
  - [`allow_dirty`](#the-allow_dirty-field) — Update dirty working directories.
  - [`changelog_config`](#the-changelog_config-field) — Path to the [git-cliff] configuration file.
  - [`changelog_update`](#the-changelog_update-field) — Update changelog.
  - [`dependencies_update`](#the-dependencies_update-field) — Update all dependencies.
  - [`git_release_enable`](#the-git_release_enable-field) — Enable git release.
  - [`git_release_draft`](#the-git_release_draft-field) — Publish git release as draft.
  - [`git_tag_enable`](#the-git_tag_enable-field) — Enable git tag.
  - [`pr_labels`](#the-pr_labels-field) — Add labels to the release Pull Request.
  - [`publish`](#the-publish-field) — Publish to cargo registry.
  - [`publish_allow_dirty`](#the-publish_allow_dirty-field) — Package dirty directories.
  - [`publish_no_verify`](#the-publish_no_verify-field) — Don't verify package build.
  - [`repo_url`](#the-repo_url-field) — Repository URL.
  - [`semver_check`](#the-semver_check-field) — Run [cargo-semver-checks].
- [`[[package]]`](#the-package-section) — Package-specific configurations.
  - [`name`](#the-name-field) — Package name. *(Required)*.
  - [`changelog_include`](#the-changelog_include-field) — Include commits from other packages.
  - [`changelog_path`](#the-changelog_path-field-package-section) — Changelog path.
  - [`changelog_update`](#the-changelog_update-field-package-section) — Update changelog.
  - [`git_release_enable`](#the-git_release_enable-field-package-section) — Enable git release.
  - [`git_release_draft`](#the-git_release_draft-field-package-section) — Publish git release as draft.
  - [`git_tag_enable`](#the-git_tag_enable-field-package-section) — Enable git tag.
  - [`publish`](#the-publish-field-package-section) — Publish to cargo registry.
  - [`publish_allow_dirty`](#the-publish_allow_dirty-field-package-section) — Package dirty directories.
  - [`publish_no_verify`](#the-publish_no_verify-field-package-section) —
  - [`semver_check`](#the-semver_check-field-package-section) — Run [cargo-semver-checks].
    Don't verify package build.

### The `[workspace]` section

Defines the global configuration, applied to all packages by default.
This section is optional, as well as all its fields.

Here's an example configuration:

```toml
[workspace]
allow_dirty = true # allow updating repositories with uncommitted changes
changelog_config = "config/git-cliff.toml"
changelog_update = false
dependencies_update = true # update dependencies with `cargo update`
git_release_enable = true
publish_allow_dirty = true
publish_no_verify = false
repo_url = "https://github.com/<owner>/<repo>"
semver_check = false
```

#### The `allow_dirty` field

- If `true`, allow release-plz to update dirty working directories.
  A directory is considered dirty if it contains uncommitted changes.
  The uncommitted changes will be part of the update.
- If `false`, release-plz returns an error if the repository contains uncommitted changes. *(Default)*.

:::caution
This field is different from the `allow-dirty` flag of the `release-plz release` command.
This field only affects the `release-plz update` and `release-plz release-pr` command.
:::

#### The `changelog_config` field

Path to the [git-cliff] configuration file.
If unspecified, release-plz uses the [keep a changelog](https://keepachangelog.com/en/1.1.0/) format.
You can learn more in the [changelog format](changelog-format.md) section.

#### The `changelog_update` field

- If `true`, update the changelog of the crates. *(Default)*.
- If `false`, don't update changelogs.

This field can be overridden in the [`[package]`](#the-package-section) section.

#### The `dependencies_update` field

- If `true`, update all the dependencies in the `Cargo.lock` file by running `cargo update`.
- If `false`, only update the workspace packages by running `cargo update --workspace`. *(Default)*.

#### The `git_release_enable` field

- If `true`, release-plz creates a git release for the created tag. *(Default)*.
- If `false`, release-plz doesn't create a git release.

The supported git releases are:

- [GitHub](https://docs.github.com/en/repositories/releasing-projects-on-github/managing-releases-in-a-repository)
- [Gitea](https://docs.gitea.io/en-us/)
- [GitLab](https://docs.gitlab.com/ee/user/project/releases/#releases)

#### The `git_release_draft` field

- If `true`, release-plz creates the git release as draft (unpublished).
- If `false`, release-plz publishes the created git release. *(Default)*.

#### The `git_tag_enable` field

- If `true`, release-plz creates a git tag for the new package version. *(Default)*.
- If `false`, release-plz doesn't create a git tag.
  Note: you can't create a git release without a git tag.

#### The `pr_labels` field

Add labels to the Pull Request opened by release-plz.
*(GitHub only)*.

#### The `publish` field

Publish to cargo registry.

- If `true`, `release-plz` runs `cargo publish`. *(Default)*.
- If `false`, `release-plz` doesn't run `cargo publish`.

With this option disabled, release-plz will continue creating git tags.
However, note that release-plz will still use the cargo registry to check what's the latest
release, so you still need to run `cargo publish` by yourself.

#### The `publish_allow_dirty` field

Allow dirty working directories to be packaged.
When `true`, `release-plz` adds the `--allow-dirty` flag to `cargo publish`.

#### The `publish_no_verify` field

Don't verify the contents by building them.
When `true`, `release-plz` adds the `--no-verify` flag to `cargo publish`.

#### The `repo_url` field

GitHub/Gitea repository URL where your project is hosted.
It is used to generate the changelog release link and open the PR.
Normally, you don't need to set this field,
because release-plz defaults to the URL of the default git remote.

#### The `semver_check` field

With this field, you can tell release-plz to run [cargo-semver-checks] to check
API breaking changes of your package:

- If `true`, run it. *(Default)*.
- If `false`, don't run it.

Note that [cargo-semver-checks] only works with packages containing a library.

This field can be overridden in the [`[package]`](#the-package-section) section.

### The `[[package]]` section

In this section, you can override some of the `workspace` fields for specific packages.

Here's an example configuration where we override the configuration of the `my_package` package:

```toml
[[package]]
name = "my_package"
changelog_path = "docs/CHANGELOG.md"
semver_check = false
changelog_update = false
git_release_enable = true
publish = true
publish_allow_dirty = true
publish_no_verify = true
```

#### The `name` field

Name of the package to which the configuration applies.
*(Required field)*.

#### The `changelog_include` field

By default, release-plz populates the changelog of a package with commits
containing changes in files of the package directory.
You can use the `changelog_include` field to include commits that belong to other packages.
For example, the changelog of the `release-plz` package of this repository
includes commits of the `release_plz_core` package, because they affect the
`release-plz` package, too.

Example:

```toml
changelog_include = ["release_plz_core"]
```

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
In GitHub Actions, this is the root of the repository.

This field cannot be set in the `[workspace]` section.

#### The `changelog_update` field (`package` section)

- If `true`, update the changelog of this package. *(Default)*.
- If `false`, don't.

#### The `git_release_enable` field (`package` section)

Overrides the [`workspace.git_release_enable`](#the-git_release_enable-field) field.

#### The `git_release_draft` field (`package` section)

Overrides the [`workspace.git_release_draft`](#the-git_release_draft-field) field.

#### The `git_tag_enable` field (`package` section)

Overrides the [`workspace.git_tag_enable`](#the-git_tag_enable-field) field.

#### The `publish` field (`package` section)

Overrides the [`workspace.publish`](#the-publish-field) field.

#### The `publish_allow_dirty` field (`package` section)

Overrides the
[`workspace.publish_allow_dirty`](#the-publish_allow_dirty-field) field.

#### The `publish_no_verify` field (`package` section)

Overrides the [`workspace.publish_no_verify`](#the-publish_no_verify-field) field.

#### The `semver_check` field (`package` section)

- If `true`, run [cargo-semver-checks] for this package.
- If `false`, don't.

By default, release-plz runs [cargo-semver-checks] if the package is a library.

[cargo-semver-checks]: https://github.com/obi1kenobi/cargo-semver-checks
[git-cliff]: https://github.com/orhun/git-cliff
