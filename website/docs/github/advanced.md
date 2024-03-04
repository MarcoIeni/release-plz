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
