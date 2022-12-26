# Release PR

With `release-plz release-pr` you can open a GitHub Pull Request that updates the version of the packages of a local workspace.

In the following example, I run `release-plz` on the `release-plz` project itself.
`Release-plz` opens a PR that increases the version and the changelog of the packages with unpublished changes.

![release-plz release-pr](https://user-images.githubusercontent.com/11428655/160772903-544c7578-7c17-4311-b6ca-a1aefeabe799.gif)

## Gitea

`release-plz release-pr` also supports creating PRs for repositories hosted on Gitea with the `--backend` flag:

`release-plz release-pr --token <gitea application token> --backend gitea`
