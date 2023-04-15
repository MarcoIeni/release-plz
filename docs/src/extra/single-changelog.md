# Single changelog

If you have a workspace with multiple packages, and you want to keep
track of the changes of just one package, you can customize your
`release-plz.toml` file like this:

```toml
[workspace]
# disable the changelog for all packages
update_changelog = false

[[package]]
name = "my-important-package"
# enable the changelog for this package
update_changelog = true
# set the path of the changelog to the root of the repository
changelog_path = "./CHANGELOG.md"
```
