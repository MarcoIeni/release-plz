# Advanced Configuration

## Git submodules

If your repository uses git submodules, set the `submodules` option in the `actions/checkout` step:

- `submodules: true` to checkout submodules.
- `submodules: recursive` to recursively checkout submodules.

For example:

```yaml
jobs:
  release-plz:
    name: Release-plz
    runs-on: ubuntu-latest
    steps:
      - name: Checkout repository
        uses: actions/checkout@v4
        with:
          fetch-depth: 0
          submodules: recursive # <-- Add this line
```

To learn more, see GitHub [docs](https://github.com/actions/checkout/).

## Add more info to commit message

By default, the commit message of the release PR only contains `chore: release`.
To add the PR title and description to the default commit message when merging a pull request,
change the GitHub repository settings under "General":

![pr settings](../assets/pr_settings.png)

You can learn more in the
[announcement](https://github.blog/changelog/2022-08-23-new-options-for-controlling-the-default-commit-message-when-merging-a-pull-request/)
and
[docs](https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/configuring-pull-request-merges/configuring-commit-squashing-for-pull-requests)
