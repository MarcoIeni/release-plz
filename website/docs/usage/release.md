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

:::info
`release-plz release` doesn't edit your `Cargo.toml` files and doesn't
push new commits. It releases the packages as they are in your repository.
For this reason, you typically use the `release-plz release` command in the main branch
after you run `release-plz update`
or you merge a pull request opened with `release-plz release-pr`.
:::

If all packages are already published, the `release-plz release` command does nothing.

To learn more, run `release-plz release --help`.

## Gitlab

`release-plz release` also supports creating releases for repositories hosted on Gitlab with
the `--backend gitlab` option:

You need to create a token in your Gitlab repo (Settings/Access Tokens) with the following
permissions:

- Role: `Maintainer` or higher
- Scopes:
  - `api` (to create a release)
  - `write_repository` (to create tag)

See the Gitlab [project access tokens](https://docs.gitlab.com/ee/user/project/settings/project_access_tokens.html)
docs.

Then you can run `release-plz release` with the following arguments:

`release-plz release --backend gitlab --git-token <gitlab_token>`

## Gitea

`releases-plz` supports creating releases on Gitea with the `--backend gitea` option.

TODO: document how to create a token on Gitea.

Then you can run `release-plz release` in Gitea CI with the following arguments:

`release-plz release --backend gitea --git-token <gitea_token>`

## Json output

You can get info about the outcome of this command by appending `-o json` to the command.
Stdout will contain info about the release:

```json
{
  "releases": [
    {
      "package_name": "<package_name>",
      "prs": "<prs>",
      "tag": "<tag_name>",
      "version": "<version>"
    }
  ]
}
```

Example:

```json
{
  "releases": [
    {
      "package_name": "my_crate",
      "prs": [
        {
          "html_url": "https://github.com/user/proj/pull/1439",
          "number": 1439
        }
      ],
      "tag": "v0.1.0",
      "version": "0.1.0"
    }
  ]
}
```

If release-plz didn't release any packages, the `releases` array will be empty.

### The `tag` field

The `tag` field is present even if the user disabled the tag creation with the
[`git_tag_enable`](../config.md#the-git_tag_enable-field) field.
This is because the user might want to use the tag name to create the tag
by themselves.

### The `prs` field

`prs` is an array of PRs present in the changelog body of the release.
Usually, they are the PRs containing the changes that were released.

Each entry of the array is an object containing:

- `html_url`: The URL of the PR.
- `number`: The number of the PR.
