# FAQ

## What packages does release-plz publish?

Release-plz publishes all packages, except:

- packages with `publish = false` in the `Cargo.toml`.
- [examples](https://doc.rust-lang.org/cargo/reference/cargo-targets.html#examples) that don't
  specify the [`publish`](https://doc.rust-lang.org/cargo/reference/manifest.html#the-publish-field)
  field in the `Cargo.toml` file. To publish them, set the field.

Even, if a package is not published, Release-plz will update its `Cargo.toml` to bump the version of
a local dependency if needed.

If you want to check which packages release-plz will publish, run
`release-plz release --dry-run`.

## Can I edit the release PR before merging it?

Yes, you can edit the release PR as you would do with any other PR.

Here are some reasons why you might want to edit the release PR:

- edit the changelog: the changelog produced by release-plz is a good starting point,
  but you might want to add more details to it.
- edit the version: if you forgot to mark a commit as a
  [breaking change](https://www.conventionalcommits.org/en/v1.0.0/#commit-message-with-description-and-breaking-change-footer),
  or if cargo-semver-checks
  [failed](https://github.com/obi1kenobi/cargo-semver-checks#will-cargo-semver-checks-catch-every-semver-violation)
  to detect a breaking change, you can manually edit the version of the package.

## Does the changelog include the commits from the whole repo?

The changelog of each crate includes the commit that changed one of the
files of the crate or one of its dependencies.

## What if a commit doesn't follow the conventional-commits format?

By default, it will be listed under the section `### Other`.
Remember you can customize the changelog format by providing a
[git-cliff](https://github.com/orhun/git-cliff) config file.
