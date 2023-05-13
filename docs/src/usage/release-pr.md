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
- otherwise, release-plz closes the old PR and opens a new one.

## Gitea

`release-plz release-pr` also supports creating PRs for repositories hosted on
Gitea with the `--backend` flag:

`release-plz release-pr --token <gitea application token> --backend gitea`

## FAQ

### Can I edit the release PR before merging it?

Yes, you can edit the release PR as you would do with any other PR.

Here are some reasons why you might want to edit the release PR:

- edit the changelog: the changelog produced by release-plz is a good starting point,
  but you might want to add more details to it.
- edit the version: if you forgot to mark a commit as a
  [breaking change](https://www.conventionalcommits.org/en/v1.0.0/#commit-message-with-description-and-breaking-change-footer),
  or if cargo-semver-checks
  [failed](https://github.com/obi1kenobi/cargo-semver-checks#will-cargo-semver-checks-catch-every-semver-violation)
  to detect a breaking change, you can manually edit the version of the package.
