# Semver check

Release plz uses [cargo-semver-checks](https://github.com/obi1kenobi/cargo-semver-checks)
to check for API breaking changes in libraries.

The check results are shown in the release Pull Request and in the output of the `release-plz update` command:

- if the check is successful, release-plz shows "(✓ API compatible changes)"
- if the check is unsuccessful, release-plz shows "(⚠️ API breaking changes)"
- if the check is skipped, release-plz shows nothing.

Example:

![pr](assets/pr.png)
