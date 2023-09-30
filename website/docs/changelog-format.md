# Changelog format

Release-plz generates the changelog by using [git-cliff](https://git-cliff.org).
By default, release-plz uses the
[keep a changelog](https://keepachangelog.com/en/1.1.0/) format.

You can customize the changelog format, by providing a git-cliff configuration
file with the `--changelog-config` argument, or with the
[`changelog_config`](config.md#the-changelog_config-field) of the configuration file.

See the [git-cliff documentation](https://git-cliff.org/docs/configuration)
to see how to customize the changelog format.

## How should I write my commits?

Release-plz assumes you are using [Conventional Commit messages](https://www.conventionalcommits.org/).

The most important prefixes you should have in mind are:

- `fix:`: represents bug fixes, and results in a [SemVer](https://semver.org/)
  patch bump.
- `feat:`: represents a new feature, and results in a SemVer minor bump.
- `<prefix>!:` (e.g. `feat!:`): represents a breaking change
  (indicated by the `!`) and results in a SemVer major bump.

Commits that don't follow the Conventional Commit format result in a SemVer patch bump.
