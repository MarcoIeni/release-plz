# Configuration

This section describes how you can customize the behavior of release-plz
with the `release-plz.toml` file.

This configuration file is optional — release-plz is designed to work out of the box,
with decent defaults.

Put your `release-plz.toml` (or `.release-plz.toml`) file in the same directory of your root `Cargo.toml`.

## Example

Here's an example configuration file for a cargo workspace.
`package_a`, `package_b` and `package_c` override some fields from the default configuration `[workspace]`,
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
publish_timeout = "10m" # set a timeout for `cargo publish`
release_commits = "^feat:" # prepare release only if at least one commit matches a regex

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
publish_features = ["a", "b"] # add `--features=a,b` to `cargo publish` for `package_b`

[[package]]
name = "package_c"
release = false # don't process this package

[changelog]
protect_breaking_commits = true # always include commits with breaking changes in the changelog
```
