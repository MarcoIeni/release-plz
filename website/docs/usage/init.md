# init

The `release-plz init` command initializes the necessary configurations and secrets for
release-plz to work properly in your GitHub repository.

Useful to initialize the GitHub action quickly.
For a complete GitHub action setup guide, check out the
[Quickstart](../github/quickstart.md) section.

:::info
Release-plz uses the [`gh` GitHub CLI](https://cli.github.com/) to store the
cargo registry token and the GitHub token in the GitHub repository secrets.
Install it before running the `release-plz init` command.
:::
