# set-version

Edit the version of a package in Cargo.toml.
Specify a version with the syntax `<package_name>@<version>`.
E.g. `release-plz set-version rand@1.2.3`

You can also set multiple versions, separated by space.
E.g. `release-plz set-version rand@1.2.3 serde@2.0.0`

:::info
This command is meant to edit the versions of the packages
of your workspace, not the version of your dependencies.
:::
