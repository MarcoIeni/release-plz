# Output

After the action runs, it outputs the following properties:

- `prs`: The release PRs opened by release-plz.
  It's an array of objects with the properties of `pr`.
  *(Not useful for now. Use `pr` instead)*.
- `pr`: The release PR opened by release-plz.
  It's a JSON object with the following properties:
  - `head_branch`: The name of the branch where the changes are implemented.
  - `base_branch`: The name of the branch the changes are pulled into.
    It is the default branch of the repository. E.g. `main`.
  - `html_url`: The URL of the PR.
  - `number`: The number of the PR.
- `releases`: The JSON output of the `release` command.
  It's an array of JSON objects with the following properties:
  - `package_name`: The name of the package that was released.
  - `tag`: git tag name of the package that was released. It's returned even if you have
    [git_tag_enable](../config.md#the-git_tag_enable-field) set to `false`, so that
    you can use this to create the git tag yourself.
  - `version`: The version of the package that was released.
- `prs_created`: Whether release-plz created any release PR. *Boolean.*
- `releases_created`: Whether release-plz released any package. *Boolean.*

## Example

```yaml
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - name: Run release-plz
        id: release-plz # <--- ID used to refer to the outputs. Don't forget it.
        uses: MarcoIeni/release-plz-action@v0.5
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
          CARGO_REGISTRY_TOKEN: ${{ secrets.CARGO_REGISTRY_TOKEN }}
      - name: Assert release
        env:
          RELEASES: ${{ steps.release-plz.outputs.releases }}
          PRS: ${{ steps.release-plz.outputs.prs }}
          PR: ${{ steps.release-plz.outputs.pr }}
          PRS_CREATED: ${{ steps.release-plz.outputs.prs_created }}
          RELEASES_CREATED: ${{ steps.release-plz.outputs.releases_created }}
        run: |
          set -e
          echo "releases: $RELEASES"
          echo "prs: $PRS"
          echo "pr: $PR"
          echo "prs_created: $PRS_CREATED"
          echo "releases_created: $RELEASES_CREATED"

          # get the number of releases with jq
          releases_length=$(echo "$RELEASES" | jq 'length')
          echo "releases_length: $releases_length"

          # access the first release with jq
          release_version=$(echo "$RELEASES" | jq -r '.[0].version')
          echo "release_version: $release_version"

          # access the first release with fromJSON. Docs: https://docs.github.com/en/actions/learn-github-actions/expressions
          echo "release_version: ${{ fromJSON(steps.release-plz.outputs.releases)[0].version }}"

          release_tag=$(echo "$RELEASES" | jq -r '.[0].tag')
          echo "release_tag: $release_tag"

          release_package_name=$(echo "$RELEASES" | jq -r '.[0].package_name')
          echo "release_package_name: $release_package_name"

          prs_length=$(echo "$PRS" | jq 'length')
          echo "prs_length: $prs_length"
```
