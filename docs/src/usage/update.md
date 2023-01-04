# Update

The `release-plz update` command updates the version and the changelog of the
packages of a local workspace.

The command:

- Downloads the packages of the project from the cargo registry.
- Compares the local packages with the downloaded ones to determine the new commits.
- Updates the packages versions based on the messages of the new commits (based on [conventional commits](https://www.conventionalcommits.org/en/v1.0.0/) and [semantic versioning](https://semver.org/)).
- Updates the packages changelogs with the messages of the new commits.
- Updates all dependencies by running `cargo update` (disabled by default).

In the following example, I run `release-plz` on the `release-plz` project itself.
`Release-plz` increases the version and the changelog of the packages with
unpublished changes.

![release-plz update](https://user-images.githubusercontent.com/11428655/160762832-54300ddb-ec9c-4538-a611-c66490c47333.gif)

To learn more, run `release-plz update --help`.
