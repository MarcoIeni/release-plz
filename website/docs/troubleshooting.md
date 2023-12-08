# Troubleshooting

## Release-plz GitHub action started misbehaving

> Did your release-plz GitHub action started misbehaving after a GitHub action [release](https://github.com/MarcoIeni/release-plz-action/releases)?

If yes, pin a specific version of the release-plz GitHub action in your workflow file.
E.g. go from `MarcoIeni/release-plz-action@v0.5` to `MarcoIeni/release-plz-action@v0.5.16`.
Determine the right version to pin by looking at the previous [releases](https://github.com/MarcoIeni/release-plz-action/releases)

Please open an [issue](https://github.com/MarcoIeni/release-plz/issues), too.

## `release-plz release` hangs

Something similar happened in [#1015](https://github.com/MarcoIeni/release-plz/issues/1015).
Try to set a low [`publish_timeout`](./config.md#the-publish_timeout-field)
in your `release-plz.toml` file to check if release-plz
is having issues to:

- check if a package was published.
- publish a package.

## See `DEBUG` logs

Release-plz uses the `RUST_LOG` environment variable to filter the level of the printed logs.
By default, release-plz shows logs at the `info` level, or more severe.
To see debug logs, use `RUST_LOG=debug release-plz`.
If you want something even more details, use `RUST_LOG=trace release-plz`
