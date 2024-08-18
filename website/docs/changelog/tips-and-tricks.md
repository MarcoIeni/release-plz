# Tips And Tricks

## Changing the group order

Use HTML comments to force them into their desired positions:

```toml
commit_parsers = [
    { message = "^feat*", group = "<!-- 0 -->New features" },
    { message = "^fix*", group = "<!-- 1 -->Bug fixes" },
    { message = "^perf*", group = "<!-- 2 -->Performance" },
    { message = "^chore*", group = "<!-- 3 -->Miscellaneous" },
]
```

This produces the following order:

- New features
- Bug fixes
- Performance
- Miscellaneous

Then strip the tags in the template with the series of filters:

```jinja2
### {{ group | striptags | trim | upper_first }}
```

## Discard duplicate commits

```jinja2
{% for commit in commits | unique(attribute="message") %}
```

## Filter merge commits

```jinja2
{% for group, commits in commits | filter(attribute="merge_commit", value=false) %}
```
