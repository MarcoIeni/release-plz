# release

The `release-plz release` command releases all the unpublished packages.

> For example, let's say you have a workspace with two packages: `pkg-a`
> (version 0.3.1) and `pkg-b` (version 0.2.2).
> The crates.io registry contains `pkg-a` version 0.3.1, but it doesn't contain
> `pkg-b` version 0.2.2 because you didn't publish this version yet.
> In this case, release-plz would release `pkg-b`.

For every release, release-plz:

- Creates a git tag named `<package_name>-v<version>` (e.g. `tokio-v1.8.1`).
- Publishes the package to the cargo registry by running `cargo publish`.
- Publishes a GitHub/Gitea/GitLab release based on the git tag.

In the tag name, `<package_name>-` is omitted if there's only one
package to publish (i.e. with `publish != false` in the `Cargo.toml` file).

Note that `release-plz release` doesn't edit your `Cargo.toml` files and doesn't
push new commits. It releases the packages as they are in your repository.
For this reason, you typically use the `release-plz release` command in the main branch
after you run `release-plz update`
or you merge a pull request opened with `release-plz release-pr`.

If all packages are already published, the `release-plz release` command does nothing.

To learn more, run `release-plz release --help`.

## Gitlab

`releases-plz` also supports creating releases on Gitlab with the `--backend gitlab` option.

The default token in CI does not have permissions to create tags, so you will need to
a custom [access token](https://docs.gitlab.com/ee/user/project/settings/project_access_tokens.html).
The permissions you need are:

- `api` (to create a release)
- `write_repository` (to create tag)

Then you can run `release-plz release` in Gitlab CI with the following arguments:

`release-plz release --backend gitlab --git-token <gitlab application token>`

## Gitea

`releases-plz` also supports creating releases on Gitea with the `--backend gitea` option.

TODO: document how to create a token on Gitea.
