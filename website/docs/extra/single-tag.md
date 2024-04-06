# Single git tag

By default, Release-plz creates a git tag for every crate that it releases.
If you want to create a single tag for all the crates in your workspace,
you can use the following
`release-plz.toml` [configuration](../config.md):

```toml
[workspace]
# Disable git releases for all packages by default
git_release_enable = false

# Disable git tags for all packages by default
git_tag_enable = false

# Options for the package I care the most, e.g. `my_main_package`.
[[package]]
name = "my_main_package"
# (Optional) Customize the git tag name to remove the `my_main_package` prefix.
git_tag_name = "v{{version}}"

# Enable git tags for this package
git_tag_enable = true

# Enable git releases for this package
git_release_enable = true
```

With this configuration, release-plz only creates the git tag when releasing `my_main_package`.
Creating git tags for the other packages is disabled
because they inherit the workspace settings.
