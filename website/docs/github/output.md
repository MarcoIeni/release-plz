# Output

After the action runs, it outputs the following properties:

- `prs`: The release PRs opened by release-plz. *(Not useful for now. Use `pr` instead)*
- `pr`: The release PR opened by release-plz.
- `releases`: The JSON output of the `release` command.
- `prs_created`: Whether release-plz created any release PR.
- `releases_created`: Whether release-plz released any package.
