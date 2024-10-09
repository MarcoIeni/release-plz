# Quickstart

This guide shows how to run the release-plz
[GitHub Action](https://github.com/marketplace/actions/release-plz)
every time you merge a commit to the main branch.
The workflow will have two jobs, running the following commands:

- [`release-plz release-pr`](../usage/release-pr.md): creates the release pr.
- [`release-plz release`](../usage/release.md): publishes the unpublished packages.

Follow the steps below to set up the GitHub Action.

## 1. Change GitHub Actions permissions

1. Go to the GitHub Actions settings:

   ![actions settings](../assets/actions_settings.png)

2. Change "Workflow permissions" to allow GitHub Actions to create and approve
   pull requests (needed to create and update the PR).

   ![workflow permission](../assets/workflow_permissions.png)

## 2. Set the `CARGO_REGISTRY_TOKEN` secret

Release-plz needs a token to publish your packages to the cargo registry.

1. Retrieve your registry token following
   [this](https://doc.rust-lang.org/cargo/reference/publishing.html#before-your-first-publish)
   guide.
2. Add your cargo registry token as a secret in your repository following
   [this](https://docs.github.com/en/actions/security-guides/encrypted-secrets#creating-encrypted-secrets-for-a-repository)
   guide.

As specified in the `cargo publish`
[options](https://doc.rust-lang.org/cargo/commands/cargo-publish.html#publish-options):

- The token for [crates.io](https://crates.io/) shall be specified with the `CARGO_REGISTRY_TOKEN`
  environment variable.
- Tokens for other registries shall be specified with environment variables of the form
  `CARGO_REGISTRIES_NAME_TOKEN` where `NAME` is the name of the registry in all capital letters.

If you are creating a new crates.io token, specify the scopes `publish-new` and `publish-update`:

![token scope](../assets/token_scope.png)

## 3. Setup the workflow

Create the release-plz workflow file under the `.github/workflows` directory
(for example `.github/workflows/release-plz.yml`)
and copy the following workflow:

```yaml
name: Release-plz

permissions:
  pull-requests: write
  contents: write

on:
  push:
    branches:
      - main

jobs:

  # Release unpublished packages.
  release-plz-release:
    name: Release-plz release
    runs-on: ubuntu-latest
    steps:
      - name: Checkout repository
        uses: actions/checkout@v4
      - name: Install Rust toolchain
        uses: dtolnay/rust-toolchain@stable
      - name: Run release-plz
        uses: MarcoIeni/release-plz-action@v0.5
        with:
          command: release
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
          CARGO_REGISTRY_TOKEN: ${{ secrets.CARGO_REGISTRY_TOKEN }}

  # Create a PR with the new versions and changelog, preparing the next release.
  release-plz-pr:
    name: Release-plz PR
    runs-on: ubuntu-latest
    concurrency:
      group: release-plz-${{ github.ref }}
      cancel-in-progress: false
    steps:
      - name: Checkout repository
        uses: actions/checkout@v4
        with:
          fetch-depth: 0
      - name: Install Rust toolchain
        uses: dtolnay/rust-toolchain@stable
      - name: Run release-plz
        uses: MarcoIeni/release-plz-action@v0.5
        with:
          command: release-pr
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
          CARGO_REGISTRY_TOKEN: ${{ secrets.CARGO_REGISTRY_TOKEN }}
```

<details>
<summary>Workflow explanation</summary>

This optional section adds comments to the above workflow,
to explain it in detail.

```yaml
# Name of the workflow: you can change it.
name: Release-plz

permissions:
  # Used to create and update pull requests.
  pull-requests: write
  # Used to push to branches, push tags, and create releases.
  contents: write

# The action runs on every push to the main branch.
on:
  push:
    branches:
      - main

jobs:

  # Release unpublished packages.
  release-plz-release:
    name: Release-plz release
    runs-on: ubuntu-latest
    steps:
      - name: Checkout repository
        uses: actions/checkout@v4
      - name: Install Rust toolchain
        uses: dtolnay/rust-toolchain@stable
      - name: Run release-plz
        uses: MarcoIeni/release-plz-action@v0.5
        with:
          command: release
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
          CARGO_REGISTRY_TOKEN: ${{ secrets.CARGO_REGISTRY_TOKEN }}

  # Create a PR with the new versions and changelog, preparing the next release.
  release-plz-pr:
    name: Release-plz PR
    runs-on: ubuntu-latest
    concurrency:
      group: release-plz-${{ github.ref }}
      cancel-in-progress: false
    steps:
      - name: Checkout repository
        uses: actions/checkout@v4
        with:
          # `fetch-depth: 0` is needed to clone all the git history, which is necessary to
          # determine the next version and build the changelog.
          # Note that it's not needed in the `release-plz-release` job.
          fetch-depth: 0
      - name: Install Rust toolchain
        uses: dtolnay/rust-toolchain@stable
      - name: Run release-plz
        uses: MarcoIeni/release-plz-action@v0.5
        with:
          command: release-pr
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
          CARGO_REGISTRY_TOKEN: ${{ secrets.CARGO_REGISTRY_TOKEN }}
```

</details>

Notes:

- The `concurrency` block guarantees that if a new commit is pushed while the job of the previous
  commit was still running, the new job will wait for the previous one to finish.
  In this way, only one instance of release-plz will run in the repository at the same time for
  the same branch, ensuring that there are no conflicts.
  See the GitHub [docs](https://docs.github.com/en/actions/writing-workflows/workflow-syntax-for-github-actions#jobsjob_idconcurrency)
  to learn more.
- `CARGO_REGISTRY_TOKEN` in the `release-plz-pr` job is only required if you
  are using a private registry.

:::info

- If you want release-plz to only update your packages,
  and you want to handle `cargo publish` and git tag push by yourself,
  remove the `release-plz-release` job.
- If you want release-plz to only release your packages,
  and you want to update `Cargo.toml` versions and changelogs by yourself.

:::

## 4. Set input variables (optional)

The GitHub action accepts the following input variables:

- `command`: The release-plz command to run. Accepted values: `release-pr`,
  `release`. (By default it runs both commands).
- `registry`: Registry where the packages are stored.
  The registry name needs to be present in the Cargo config.
  If unspecified, the `publish` field of the package manifest is used.
  If the `publish` field is empty, crates.io is used.
- `manifest_path`: Path to the Cargo.toml of the project you want to update.
  Both Cargo workspaces and single packages are supported. (Defaults to the root
  directory).
- `version`: Release-plz version to use. E.g. `0.3.70`. (Default: latest version).
- `config`: Release-plz config file location. (Defaults to
  `release-plz.toml` or `.release-plz.toml`).
- `token`: Token used to publish to the cargo registry.
  Override the `CARGO_REGISTRY_TOKEN` environment variable, or the `CARGO_REGISTRIES_<NAME>_TOKEN`
  environment variable, used for registry specified in the `registry` input variable.
- `backend`: Forge backend. Valid values: `github`, `gitea`. (Defaults to `github`).

You can specify the input variables by using the `with` keyword.
For example:

```yaml
steps:
  - ...
  - name: Run release-plz
    uses: MarcoIeni/release-plz-action@v0.5
    with: # <--- Input variables
      command: release-pr
      registry: my-registry
      manifest_path: rust-crates/my-crate/Cargo.toml
      version: 0.3.70
    env:
      GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
      CARGO_REGISTRY_TOKEN: ${{ secrets.CARGO_REGISTRY_TOKEN }}
```
