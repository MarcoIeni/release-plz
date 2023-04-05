# Yanked packages

Due to [this](https://github.com/rust-lang/cargo/issues/11693)
issue, release-plz can't detect yanked packages.

If you run release-plz on a package that has been yanked, you will see the message:

```txt
the local package has already a different version with respect to the registry package, so release-plz will not update it
```

This is because release-plz thinks that the latest published version of your packages is the latest
non-yanked version.

If the current version of your package is yanked, please update the version of your package to the next version manually.
You can use the `cargo set-version` command from [cargo-edit](https://github.com/killercup/cargo-edit).
