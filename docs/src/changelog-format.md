# Changelog format

Release-plz generates the changelog by using [git-cliff](https://github.com/orhun/git-cliff).
By default, release-plz uses the
[keep a changelog](https://keepachangelog.com/en/1.1.0/) format.
You can customize the changelog format, by providing a git-cliff configuration
file with the `--changelog-config` argument, or with the
[`changelog_config`](config.md#the-changelog_config-field) of the configuration file.
