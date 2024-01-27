# Changelog format

Release-plz generates the changelog by using [git-cliff](https://git-cliff.org) as a library.
By default, release-plz uses the
[keep a changelog](https://keepachangelog.com/en/1.1.0/) format.

You can customize the changelog format in the [`[changelog]`](./config.md#the-changelog-section)  section of the configuration.

## How should I write my commits?

Release-plz assumes you are using [Conventional Commit messages](https://www.conventionalcommits.org/).

The most important prefixes you should have in mind are:

- `fix:`: represents bug fixes, and results in a [SemVer](https://semver.org/)
  patch bump.
- `feat:`: represents a new feature, and results in a SemVer minor bump.
- `<prefix>!:` (e.g. `feat!:`): represents a breaking change
  (indicated by the `!`) and results in a SemVer major bump.

Commits that don't follow the Conventional Commit format result in a SemVer patch bump.

## Body template

A template is a text where variables and expressions get replaced with values when it is rendered.
By providing a custom [`body`](./config.md#the-body-field) template, you can customize the changelog format.

TODO: include https://git-cliff.org/docs/templating

### Syntax

**git-cliff** uses the [Tera](https://keats.github.io/tera/) template engine.

See the [Tera Documentation](https://keats.github.io/tera/docs/#templates) for more information about [control structures](https://keats.github.io/tera/docs/#control-structures), [built-ins filters](https://keats.github.io/tera/docs/#built-ins), etc.

Custom built-in filters that **git-cliff** uses:

- `upper_first`: Converts the first character of a string to uppercase.

### Context

TODO: finish this https://git-cliff.org/docs/templating/context

The context is the model that holds the required data for a template rendering. The [JSON](https://en.wikipedia.org/wiki/JSON) format is used in the following examples for the representation of a context.

...


### Examples

[Here](https://git-cliff.org/docs/templating/examples) you can find some examples of custom git-cliff templates.
Converting the git-cliff configuration file into the [`[changelog]`](./config.md#the-changelog-section) section of the release-plz configuration file is easy.

If you want to contribute your cool template using the release-plz configuration file, please open a PR! 🙏

### Tips and tricks

https://git-cliff.org/docs/tips-and-tricks
