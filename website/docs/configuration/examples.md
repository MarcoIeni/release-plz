# Examples

See the
[examples](https://github.com/MarcoIeni/release-plz/tree/main/examples)
directory for configuration files. All examples based on the following [Git
history](https://github.com/orhun/git-cliff-readme-example):

```log
* df6aef4 (HEAD -> master) feat(cache): use cache while fetching pages
* a9d4050 feat(config): support multiple file formats
* 06412ac (tag: v1.0.1) chore(release): add release script
* e4fd3cf refactor(parser): expose string functions
* ad27b43 (tag: v1.0.0) docs(example)!: add tested usage example
* 9add0d4 fix(args): rename help argument due to conflict
* a140cef feat(parser): add ability to parse arrays
* 81fbc63 docs(project): add README.md
* a78bc36 Initial commit
```

If you want to contribute your cool template using the `release-plz`
configuration file, [please open a
PR](https://github.com/MarcoIeni/release-plz/blob/main/CONTRIBUTING.md)! 🙏

## [Keep a Changelog](https://github.com/MarcoIeni/release-plz/tree/main/examples/keepachangelog.toml)

<details>
  <summary>Raw Output</summary>

```text
# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Support multiple file formats

### Changed

- Use cache while fetching pages

## [1.0.1] - 2021-07-18

### Added

- Add release script

### Changed

- Expose string functions

## [1.0.0] - 2021-07-18

### Added

- Add README.md
- Add ability to parse arrays
- Add tested usage example

### Fixed

- Rename help argument due to conflict

[unreleased]: https://github.com/orhun/git-cliff-readme-example/compare/v1.0.1..HEAD
[1.0.1]: https://github.com/orhun/git-cliff-readme-example/compare/v1.0.0..v1.0.1
```

</details>

<details>
  <summary>Rendered Output</summary>

```md
# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Support multiple file formats

### Changed

- Use cache while fetching pages

## [1.0.1] - 2021-07-18

### Added

- Add release script

### Changed

- Expose string functions

## [1.0.0] - 2021-07-18

### Added

- Add README.md
- Add ability to parse arrays
- Add tested usage example

### Fixed

- Rename help argument due to conflict

[unreleased]: https://github.com/orhun/git-cliff-readme-example/compare/v1.0.1..HEAD
[1.0.1]: https://github.com/orhun/git-cliff-readme-example/compare/v1.0.0..v1.0.1
```

</details>

## [Styled and scoped](https://github.com/MarcoIeni/release-plz/tree/main/examples/styled-scoped.toml)

<details>
  <summary>Raw Output</summary>

```text
# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### ⛰️  Features

- feat(config): support multiple file formats ([a9d4050](a9d4050212a18f6b3bd76e2e41fbb9045d268b80))
- feat(cache): use cache while fetching pages ([df6aef4](df6aef41292f3ffe5887754232e6ea7831c50ba5))

## [1.0.1](https://github.com/orhun/git-cliff-readme-example/compare/v1.0.0...v1.0.1)

### 🚜 Refactor

- refactor(parser): expose string functions ([e4fd3cf](e4fd3cf8e2e6f49c0b57f66416e886c37cbb3715))

### ⚙️ Miscellaneous Tasks

- chore(release): add release script ([06412ac](06412ac1dd4071006c465dde6597a21d4367a158))

## [1.0.0] - 2021-07-18

### ⛰️  Features

- feat(parser): add ability to parse arrays ([a140cef](a140cef0405e0bcbfb5de44ff59e091527d91b38))

### 🐛 Bug Fixes

- fix(args): rename help argument due to conflict ([9add0d4](9add0d4616dc95a6ea8b01d5e4d233876b6e5e00))

### 📚 Documentation

- docs(project): add README.md ([81fbc63](81fbc6365484abf0b4f4b05d384175763ad8db44))
- docs(example)!: add tested usage example ([ad27b43](ad27b43e8032671afb4809a1a3ecf12f45c60e0e))
```

</details>

<details>
  <summary>Rendered Output</summary>

```md
# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### ⛰️  Features

- feat(config): support multiple file formats (a9d4050)
- feat(cache): use cache while fetching pages (df6aef4)

## [1.0.1](https://github.com/orhun/git-cliff-readme-example/compare/v1.0.0...v1.0.1)

### 🚜 Refactor

- refactor(parser): expose string functions (e4fd3cf)

### ⚙️ Miscellaneous Tasks

- chore(release): add release script (06412ac)

## [1.0.0] - 2021-07-18

### ⛰️  Features

- feat(parser): add ability to parse arrays (a140cef)

### 🐛 Bug Fixes

- fix(args): rename help argument due to conflict (9add0d4)

### 📚 Documentation

- docs(project): add README.md (81fbc63)
- docs(example)!: add tested usage example (ad27b43)
```

</details>


## [Detailed](https://github.com/MarcoIeni/release-plz/tree/main/examples/detailed.toml)

<details>
  <summary>Raw Output</summary>

```text
# Changelog

All notable changes to this project will be documented in this file.

## [unreleased]

### Features

- Support multiple file formats ([a9d4050](a9d4050212a18f6b3bd76e2e41fbb9045d268b80))
- Use cache while fetching pages ([df6aef4](df6aef41292f3ffe5887754232e6ea7831c50ba5))

## [1.0.1] - 2021-07-18

[ad27b43](ad27b43e8032671afb4809a1a3ecf12f45c60e0e)...[06412ac](06412ac1dd4071006c465dde6597a21d4367a158)

### Miscellaneous Tasks

- Add release script ([06412ac](06412ac1dd4071006c465dde6597a21d4367a158))

### Refactor

- Expose string functions ([e4fd3cf](e4fd3cf8e2e6f49c0b57f66416e886c37cbb3715))

## [1.0.0] - 2021-07-18

### Bug Fixes

- Rename help argument due to conflict ([9add0d4](9add0d4616dc95a6ea8b01d5e4d233876b6e5e00))

### Documentation

- Add README.md ([81fbc63](81fbc6365484abf0b4f4b05d384175763ad8db44))
- Add tested usage example ([ad27b43](ad27b43e8032671afb4809a1a3ecf12f45c60e0e))

### Features

- Add ability to parse arrays ([a140cef](a140cef0405e0bcbfb5de44ff59e091527d91b38))
```

</details>

<details>
  <summary>Rendered Output</summary>

```md
# Changelog

All notable changes to this project will be documented in this file.

## [unreleased]

### Features

- Support multiple file formats (a9d4050)
- Use cache while fetching pages (df6aef4)

## [1.0.1] - 2021-07-18

ad27b43...06412ac

### Miscellaneous Tasks

- Add release script (06412ac)

### Refactor

- Expose string functions (e4fd3cf)

## [1.0.0] - 2021-07-18

### Bug Fixes

- Rename help argument due to conflict (9add0d4)

### Documentation

- Add README.md (81fbc63)
- Add tested usage example (ad27b43)

### Features

- Add ability to parse arrays (a140cef)
```

</details>
