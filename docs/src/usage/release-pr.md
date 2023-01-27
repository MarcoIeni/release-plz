# Release PR

The `release-plz release-pr` command runs [`release-plz update`](update.md) and
opens a GitHub Pull Request that prepares the next release.

When the project maintainer merges the release pull request, the packages are
ready to be published.

Here's an example of a [PR](https://github.com/MarcoIeni/release-plz/pull/377)
opened by release-plz in the release-plz GitHub project itself.

In the following example, I run `release-plz` on the `release-plz` project
itself.
`Release-plz` opens a PR that increases the version and the changelog of the
packages with unpublished changes.

![release-plz release-pr](https://user-images.githubusercontent.com/11428655/160772903-544c7578-7c17-4311-b6ca-a1aefeabe799.gif)

To learn more, run `release-plz release-pr --help`.

## Pr update

If there's already an open release PR:

- if the PR contains commits that are not from bots (except the first one),
  release-plz closes the PR to preserve the git history.
  (Not available on Gitea).
- otherwise, release-plz closes the old PR and opens a new one

## Gitea

`release-plz release-pr` also supports creating PRs for repositories hosted on
Gitea with the `--backend` flag:

`release-plz release-pr --token <gitea application token> --backend gitea`
