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
  - `prs`: Array of PRs present in the changelog body of the release.
    Usually, they are the PRs containing the changes that were released.
    Each entry is an object containing:
    - `html_url`: The URL of the PR.
    - `number`: The number of the PR.
  - `tag`: git tag name of the package that was released. It's returned even if you have
    [git_tag_enable](../config.md#the-git_tag_enable-field) set to `false`, so that
    you can use this to create the git tag yourself.
  - `version`: The version of the package that was released.
- `prs_created`: Whether release-plz created any release PR. *Boolean.*
- `releases_created`: Whether release-plz released any package. *Boolean.*

## Example: read the output

```yaml
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - name: Checkout repository
        uses: actions/checkout@v4
        with:
          fetch-depth: 0
      - name: Install Rust toolchain
        uses: dtolnay/rust-toolchain@stable
      - name: Run release-plz
        id: release-plz # <--- ID used to refer to the outputs. Don't forget it.
        uses: MarcoIeni/release-plz-action@v0.5
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
          CARGO_REGISTRY_TOKEN: ${{ secrets.CARGO_REGISTRY_TOKEN }}
      - name: Read release output
        env:
          RELEASES: ${{ steps.release-plz.outputs.releases }}
          PRS: ${{ steps.release-plz.outputs.prs }}
          PR: ${{ steps.release-plz.outputs.pr }}
          PRS_CREATED: ${{ steps.release-plz.outputs.prs_created }}
          RELEASES_CREATED: ${{ steps.release-plz.outputs.releases_created }}
        run: |
          set -e
          echo "releases: $RELEASES" # example: [{"package_name":"my-package","prs":[{"html_url":"https://github.com/user/proj/pull/1439","number":1439}],"tag":"v0.1.0","version":"0.1.0"}]
          echo "prs: $PRS" # example: [{"base_branch":"main","head_branch":"release-plz-2024-05-01T20-38-05Z","html_url":"https://github.com/MarcoIeni/rust-workspace-example/pull/198","number":198}]
          echo "pr: $PR" # example: {"base_branch":"main","head_branch":"release-plz-2024-05-01T20-38-05Z","html_url":"https://github.com/MarcoIeni/rust-workspace-example/pull/198","number":198}
          echo "prs_created: $PRS_CREATED" # example: true
          echo "releases_created: $RELEASES_CREATED" # example: true

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

          # print all names of released packages, one per line
          echo "package_names: $(echo "$RELEASES" | jq -r '.[].package_name')"
          # TODO: show how to store this in a variable and iterate over it (maybe an array?). PR welcome!

          # iterate over released packages
          for package_name in $(echo "$RELEASES" | jq -r '.[].package_name'); do
              echo "released $package_name"
          done

          echo "pr_number: ${{ fromJSON(steps.release-plz.outputs.pr).number }}"
          echo "pr_html_url: ${{ fromJSON(steps.release-plz.outputs.pr).html_url }}"
          echo "pr_head_branch: ${{ fromJSON(steps.release-plz.outputs.pr).head_branch }}"
          echo "pr_base_branch: ${{ fromJSON(steps.release-plz.outputs.pr).base_branch }}"
```
