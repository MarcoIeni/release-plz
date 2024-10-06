# Busy repositories

This section discusses some considerations when using release-plz in busy repositories.
For the purpose of this document, a busy repository is a repository with the following characteristics:

- The CI runs in parallel on multiple commits of the main branch
  either because the CI is slow or because the repository is very active.
- The repository is maintained by multiple developers that might
  push commits to the main branch while `release-plz release` is running (typically, a solo maintainer
  waits for the release to finish before pushing other commits to the repo).

If you are using release-plz in a busy repository, please read this section carefully.

## Concurrency

Release-plz [docs](../github/quickstart.md) suggest using the GitHub Actions
`concurrency` block like this to prevent multiple release-plz jobs from running at the same time:

```yaml
concurrency:
  group: release-plz-${{ github.ref }}
  cancel-in-progress: false
```

However, if you have [release_always](../config.md#the-release_always-field) set to `false`, your release will be skipped
if release-plz is already running for a previous commit and a new commit is pushed after the release PR is merged.

This is an example commit sequence where the release is skipped:

- Commit 1: an initial commit is pushed to the main branch. Release-plz runs.
- Commit 2: a second commit is pushed to the main branch. The job of this commit is pending,
  waiting for Release-plz to finish on Commit 1.
- Commit 3: a third commit is pushed to the main branch. The job of commit 2 is canceled,
  and the job of commit 3 is pending, waiting for Release-plz to finish on Commit 1.

If this is a concern, you might want to have two separate workflows:

- One that runs [release-plz release](../github/quickstart.md#example-release-only) on every commit to the main branch
  *without* the `concurrency` block.
- One that runs [release-plz release-pr](../github/quickstart.md#example-release-pr-only)
  on every commit to the main branch *with* the `concurrency` block.
