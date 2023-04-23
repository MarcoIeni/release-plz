# Usage

Release-plz updates the versions and changelogs of your rust packages, by
analyzing your git history:

- [`release-plz update`](update.md) updates your project locally, without
  committing any change.
- [`release-plz release-pr`](release-pr.md) opens a GitHub Pull Request.

Once the changes are merged to the main branch, you can use
[`release-plz release`](release.md) to publish the new versions of the packages.

To learn more about how to use release-plz, run `release-plz --help`.

## FAQ

### Are packages with `publish = false` in the `Cargo.toml` updated?

Release-plz ignores packages with `publish = false` in the `Cargo.toml`.
Release-plz only updates their `Cargo.toml` to bump the version of
a local dependency when needed.
