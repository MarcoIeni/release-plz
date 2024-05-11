# Single changelog

## One package

If you have a workspace with multiple packages, and you want to keep
track of the changes of just one package, you can customize your
`release-plz.toml` file like this:

```toml
[workspace]
# disable the changelog for all packages
changelog_update = false

[[package]]
name = "my-important-package"
# enable the changelog for this package
changelog_update = true
# set the path of the changelog to the root of the repository
changelog_path = "./CHANGELOG.md"
```

To include commits of other packages in the changelog of
your main package, use the [changelog_include](../config.md#the-changelog_include-field) field.

## All packages

If you have a workspace with multiple packages, and you want to group all the
changes in a single changelog, you can customize your `release-plz.toml`
file like this:

<!--
TODO: allow to set the `changelog_path` at workspace level

```toml
[workspace]
# set the path of all the crates to the changelog to the root of the repository
changelog_path = "./CHANGELOG.md"
```

If you only cares about a few packages of the workspace: -->

```toml
[workspace]
# disable the changelog for all packages
changelog_update = false

[[package]]
name = "package_a"
# enable the changelog for this package
changelog_update = true
changelog_path = "./CHANGELOG.md"

[[package]]
name = "package_b"
changelog_update = true
changelog_path = "./CHANGELOG.md"

[changelog]
body = """

## `{{ package }}` - [{{ version | trim_start_matches(pat="v") }}](https://github.com/me/my-proj/compare/{{ package }}-v{{ previous.version }}...{{ package }}-v{{ version }}) - {{ timestamp | date(format="%Y-%m-%d") }}
{% for group, commits in commits | group_by(attribute="group") %}
### {{ group | upper_first }}
{% for commit in commits %}
{%- if commit.scope -%}
- *({{commit.scope}})* {% if commit.breaking %}[**breaking**] {% endif %}{{ commit.message }}{%- if commit.links %} ({% for link in commit.links %}[{{link.text}}]({{link.href}}) {% endfor -%}){% endif %}
{% else -%}
- {% if commit.breaking %}[**breaking**] {% endif %}{{ commit.message }}
{% endif -%}
{% endfor -%}
{% endfor %}
"""
```

The difference with the default changelog body configuration is that the header now also contains
the `{{package}}`.

In this way, `package_a` and `package_b` changelogs are in the same file.
Note that the changelog will contain duplicate changes.
If you want to merge updates of different packages into one, check
the [changelog_include](../config.md#the-changelog_include-field) field.
