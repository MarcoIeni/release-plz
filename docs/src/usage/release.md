# Release

The `release-plz release` command releases all the unpublished packages.

> For example, let's say you have a workspace with two packages: `pkg-a`
> (version 0.3.1) and `pkg-b` (version 0.2.2).
> The crates.io registry contains `pkg-a` version 0.3.1, but it doesn't contain
> `pkg-b` version 0.2.2 because you didn't publish this version yet.
> In this case, release-plz would release `pkg-b`.

For every release, release-plz:

- Creates a git tag named `<package_name>-v<version>` (e.g. `tokio-v1.8.1`).
- Publishes the package to the cargo registry by running `cargo publish`.
- Publishes a GitHub/Gitea release based on the git tag.

In the tag name, `<package_name>-` is omitted if there's only one
package to publish (i.e. with `publish != flase` in the `Cargo.toml` file).

Usually, you use the `release-plz release` command in the main branch,
after you run `release-plz update`
or you merge a pull request opened with `release-plz release-pr`.

To learn more, run `release-plz release --help`.
