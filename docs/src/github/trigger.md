# Triggering further workflow runs

Pull requests created by GitHub actions using the default `GITHUB_TOKEN` cannot
trigger other workflows.
If you have `on: pull_request` or `on: push` workflows acting as checks on pull
requests, they will not run.

> When you use the repository's `GITHUB_TOKEN` to perform tasks, events triggered
by the `GITHUB_TOKEN` will not create a new workflow run.
This prevents you from accidentally creating recursive workflow runs.
For example, if a workflow run pushes code using the repository's `GITHUB_TOKEN`,
a new workflow will not run even when the repository contains a workflow
configured to run when `push` events occur.

-- [GitHub Actions: Triggering a workflow from a workflow](https://docs.github.com/en/actions/using-workflows/triggering-a-workflow#triggering-a-workflow-from-a-workflow)

## Workarounds to trigger further workflow runs

Release-plz can release your packages without triggering further workflow runs.
However, if you want to run CI checks on the release PR,
or if you want to trigger another workflow after release-plz pushes
a tag or creates a release, you can use one of these workarounds:

- To run `on: pull_request` workflows, manually close and reopen the release pull request.

- [Personal Access Token (PAT)](https://docs.github.com/en/github/authenticating-to-github/creating-a-personal-access-token)
  created on an account with write access to the repository.
  This is the standard workaround
  [recommended by GitHub](https://docs.github.com/en/actions/using-workflows/triggering-a-workflow#triggering-a-workflow-from-a-workflow).
  Note that the account that owns the PAT will be the author of the release pull request.
  PAT works with:
  There are two types of PAT:
  - classic: less secure because you can't scope it to a single repository. Release-plz needs `repo` permissions:
    ![](../assets/pat-classic.png)
  - fine-grained: more secure because you can select the repositories where the PAT can be used. Release-plz needs the following:
    - Select the repositories where you want to use the PAT, to give it write access:
      ![](../assets/repository-access.png)
    - Assign "Contents" and "Pull requests" read and write permissions:
      ![](../assets/pat-overview.png)
- Use [SSH (deploy keys)](https://github.com/peter-evans/create-pull-request/blob/main/docs/concepts-guidelines.md#push-using-ssh-deploy-keys)
  to push the pull request branch.
  Note that this method will only trigger `on: push` workflows.

- Use a
  [GitHub App to generate a token](https://github.com/peter-evans/create-pull-request/blob/main/docs/concepts-guidelines.md#authenticating-with-github-app-generated-tokens)
  that can be used with this action. This is the approach used by the
  [release-plz](https://github.com/MarcoIeni/release-plz/blob/main/.github/workflows/release-plz.yml)
  repo itself.
  If you want to use the release-plz logo for the GitHub app, you can find it [here](../assets/robot_head.jpeg).

In any case, pass your GitHub token to both the `actions/checkout` and `release-plz` actions:

```yaml
jobs:
  release-plz:
    name: Release-plz
    runs-on: ubuntu-latest
    steps:
      - name: Checkout repository
        uses: actions/checkout@v3
        with:
          fetch-depth: 0
          token: ${{ secrets.MY_GITHUB_TOKEN }} # <-- Your token here
      - name: Install Rust toolchain
        uses: dtolnay/rust-toolchain@stable
      - name: Run release-plz
        uses: MarcoIeni/release-plz-action@v0.5
        env:
          GITHUB_TOKEN: ${{ secrets.MY_GITHUB_TOKEN }} # <-- Your token here
          CARGO_REGISTRY_TOKEN: ${{ secrets.CARGO_REGISTRY_TOKEN }}
```

## Examples of workflows that can be triggered
- When a release is published:

  ```yaml
  on:
    release:
      types: [published]
  ```

- When a tag is pushed:

  ```yaml
  on:
    push:
      tags:
        - "*"
   ```

## Credits

This section is inspired by
[this](https://github.com/peter-evans/create-pull-request/blob/main/docs/concepts-guidelines.md#triggering-further-workflow-runs)
guide.
