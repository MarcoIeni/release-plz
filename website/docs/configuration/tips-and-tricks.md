# Tips And Tricks

## Grouping commits by category

Grouping commits can be achieved with
[`commit_parsers`](./reference.md#the-commit_parsers-field). Be mindful of
[how you write your commits](./changelog.md#how-should-i-write-my-commits).

```toml
commit_parsers = [
  { message = "^feat", group = "<!-- 0 -->⛰️ Features" },
  { message = "^fix", group = "<!-- 1 -->🐛 Bug Fixes" },
  { message = "^doc", group = "<!-- 3 -->📚 Documentation" },
  { message = "^perf", group = "<!-- 4 -->⚡ Performance" },
  { message = "^refactor\\(clippy\\)", skip = true },
  { message = "^refactor", group = "<!-- 2 -->🚜 Refactor" },
  { message = "^style", group = "<!-- 5 -->🎨 Styling" },
  { message = "^test", group = "<!-- 6 -->🧪 Testing" },
  { message = "^chore\\(release\\):", skip = true },
  { message = "^chore: release", skip = true },
  { message = "^chore\\(deps.*\\)", skip = true },
  { message = "^chore\\(pr\\)", skip = true },
  { message = "^chore\\(pull\\)", skip = true },
  { message = "^chore\\(npm\\).*yarn\\.lock", skip = true },
  { message = "^chore|^ci", group = "<!-- 7 -->⚙️ Miscellaneous Tasks" },
  { body = ".*security", group = "<!-- 8 -->🛡️ Security" },
  { message = "^revert", group = "<!-- 9 -->◀️ Revert" },
]
```

### Changing the group order

Use HTML comments to force them into their desired positions:

```toml
commit_parsers = [
    { message = "^feat*", group = "<!-- 0 -->🚀 New features" },
    { message = "^fix*", group = "<!-- 1 -->🐛 Bug fixes" },
    { message = "^perf*", group = "<!-- 2 -->⚡ Performance" },
    { message = "^chore*", group = "<!-- 3 -->⚙️ Miscellaneous" },
]
```

This produces the following order:

- 🚀 New features
- 🐛 Bug fixes
- ⚡ Performance
- ⚙️ Miscellaneous

Then strip the tags in the template with the series of filters:

```jinja2
### {{ group | striptags | trim | upper_first }}
```

### Use GitHub PR labels as groups

```toml
commit_parsers = [
  { field = "github.pr_labels", pattern = "breaking-change", group = "<!-- 0 --> 🏗️ Breaking changes" },
  { field = "github.pr_labels", pattern = "type/enhancement", group = "<!-- 1 --> 🚀 Features" },
  { field = "github.pr_labels", pattern = "type/bug", group = "<!-- 2 --> 🐛 Fixes" },
  { field = "github.pr_labels", pattern = "type/update", group = "<!-- 3 --> 🧪 Dependencies" },
  { field = "github.pr_labels", pattern = "type/refactor", group = "<!-- 4 --> 🏭 Refactor" },
  { field = "github.pr_labels", pattern = "area/documentation", group = "<!-- 5 --> 📝 Documentation" },
  { field = "github.pr_labels", pattern = ".*", group = "<!-- 6 --> 🌀 Miscellaneous" },
]
```

## Discard duplicate commits

```jinja2
{% for commit in commits | unique(attribute="message") %}
```

## Filter merge commits

```jinja2
{% for group, commits in commits | filter(attribute="merge_commit", value=false) %}
```

## Skip commits with an empty body

```toml
commit_parsers = [
  { body = "$^", skip = true },
]
```
