# Introduction

Release-plz updates the versions and changelogs of your rust packages, by analyzing your git history,
based on [conventional commits](https://www.conventionalcommits.org/en/v1.0.0/):

- `release-plz update` updates your project locally, without committing any change.
- `release-plz release-pr` opens a GitHub Pull Request.

Once the changes are merged to the main branch, you can use
`release-plz release` to publish the new versions of the packages.

![release-plz release-pr](https://user-images.githubusercontent.com/11428655/160772903-544c7578-7c17-4311-b6ca-a1aefeabe799.gif)
