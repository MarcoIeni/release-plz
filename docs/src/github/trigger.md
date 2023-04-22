# Triggering further workflow runs

Pull requests created by GitHub actions using the default `GITHUB_TOKEN` cannot
trigger other workflows.
For example, `on: pull_request` or `on: push` workflows acting as checks on pull
requests won't run.

You can learn more in the GitHub
[docs](https://docs.github.com/en/actions/using-workflows/triggering-a-workflow#triggering-a-workflow-from-a-workflow).

## Workarounds to trigger further workflow runs

Release-plz doesn't need to trigger further workflow runs to release your packages.
However, if you want to run CI checks on the release PR,
or if you want to trigger another workflow after release-plz pushes
a tag or creates a release, you need to use one of these workarounds:

- To run `on: pull_request` workflows, manually close and reopen the release pull request.

- Use a [Personal Access Token (PAT)](https://docs.github.com/en/github/authenticating-to-github/creating-a-personal-access-token)
  created on an account with write access to the repository.
  This is the standard workaround
  [recommended by GitHub](https://docs.github.com/en/actions/using-workflows/triggering-a-workflow#triggering-a-workflow-from-a-workflow).
  Note that the account that owns the PAT will be the author of the release pull request.
  There are two types of PAT:
  - [Classic](https://docs.github.com/en/authentication/keeping-your-account-and-data-secure/creating-a-personal-access-token#personal-access-tokens-classic):
    less secure because you can't scope it to a single repository.
    Release-plz needs `repo` permissions:
    ![pat classic permissions](../assets/pat-classic.png)
  - [Fine-grained](https://docs.github.com/en/authentication/keeping-your-account-and-data-secure/creating-a-personal-access-token#fine-grained-personal-access-tokens):
    more secure because you can select the repositories where the PAT can be used.
    Release-plz needs the following:
    - Select the repositories where you want to use the PAT, to give it write access:
      ![pat repository access](../assets/repository-access.png)
    - Assign "Contents" and "Pull requests" read and write permissions:
      ![pat fine permissions](../assets/pat-overview.png)

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

## How to trigger further workflows

You can trigger workflows on different
[events](https://docs.github.com/en/actions/using-workflows/events-that-trigger-workflows).
For example:

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
