# Semver check

Release plz uses [cargo-semver-checks](https://github.com/obi1kenobi/cargo-semver-checks)
to check for API breaking changes in libraries.

The check results are shown in the release Pull Request and in the output of the
`release-plz update` command:

- If the check is skipped, release-plz shows nothing. This happens when the package
  doesn't contain a library.
- If the check is successful, release-plz shows "(✓ API compatible changes)".
- If the check is unsuccessful, release-plz shows "(⚠️ API breaking changes)", with a report
  of what went wrong.

Example:

![pr](assets/pr.png)

You can configure whether to run the check or not in the [configuration file](config.md#the-semver_check-field).

## FAQ

## What's an API breaking change?

It is a change that makes the new version of your library
incompatible with the previous one.

For example, renaming a public function of your library is an API breaking change,
because the users of your library will have to update their code to use the new name.

## Will cargo-semver-checks catch every semver violation?

No, it won't. There are many ways to break semver, and cargo-semver-checks [doesn't yet have lints for all of them](https://github.com/obi1kenobi/cargo-semver-checks/issues/5).
