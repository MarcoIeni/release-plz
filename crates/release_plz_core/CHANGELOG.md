# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.5.6](https://github.com/MarcoIeni/release-plz/compare/release_plz_core-v0.5.5...release_plz_core-v0.5.6) - 2023-03-17

### Added
- detect git remote (#610)

### Fixed
- do git fetch before rebase (#607)

## [0.5.5](https://github.com/MarcoIeni/release-plz/compare/release_plz_core-v0.5.4...release_plz_core-v0.5.5) - 2023-03-13

### Added
- write changelog in pr body (#598)

## [0.5.4](https://github.com/MarcoIeni/release-plz/compare/release_plz_core-v0.5.3...release_plz_core-v0.5.4) - 2023-03-10

### Fix
- update to cargo v0.69 to be compatible with sparse index.

## [0.5.3](https://github.com/MarcoIeni/release-plz/compare/release_plz_core-v0.5.2...release_plz_core-v0.5.3) - 2023-03-09

### Added
- include version in pr title for single crate (#593)

### Other
- retry failing http calls (#585)

## [0.5.2](https://github.com/MarcoIeni/release-plz/compare/release_plz_core-v0.5.1...release_plz_core-v0.5.2) - 2023-03-04

### Added
- detect circular dependency (#581)

## [0.5.1](https://github.com/MarcoIeni/release-plz/compare/release_plz_core-v0.5.0...release_plz_core-v0.5.1) - 2023-02-27

### Fixed
- remove ansi escape sequences in cargo-semver-checks output (#575)

## [0.5.0](https://github.com/MarcoIeni/release-plz/compare/release_plz_core-v0.4.21...release_plz_core-v0.5.0) - 2023-02-26

### Added
- Add cargo-semver-checks integration. If the `cargo-semver-checks` binary is present, release-plz uses
  it to check semver compatibility. If `cargo-semver-checks` detects an API breaking change, release-plz
  updates the major version. (#568)

### Fixed
- when editing a release-pr, update pr title and body (#571)

## [0.4.21](https://github.com/MarcoIeni/release-plz/compare/release_plz_core-v0.4.20...release_plz_core-v0.4.21) - 2023-02-20

### Other
- remove unused check (#559)

## [0.4.20](https://github.com/MarcoIeni/release-plz/compare/release_plz_core-v0.4.19...release_plz_core-v0.4.20) - 2023-02-18

### Fixed
- *(release)* trust gh workspace (#553)

## [0.4.19](https://github.com/MarcoIeni/release-plz/compare/release_plz_core-v0.4.18...release_plz_core-v0.4.19) - 2023-02-18

### Other
- `release-plz release` creates a release in Gitea, too (#539)

## [0.4.18](https://github.com/MarcoIeni/release-plz/compare/release_plz_core-v0.4.17...release_plz_core-v0.4.18) - 2023-02-11

### Added
- *(release)* add the possibility to add `--no-verify` and `--allow-dirty` as cargo publish flags (#532)

## [0.4.17](https://github.com/MarcoIeni/release-plz/compare/release_plz_core-v0.4.16...release_plz_core-v0.4.17) - 2023-02-10

### Added
- update pr in gitea (#530)

### Fixed
- update branch from main before updating PR (#528)

## [0.4.16](https://github.com/MarcoIeni/release-plz/compare/release_plz_core-v0.4.15...release_plz_core-v0.4.16) - 2023-02-08

### Added
- add changelog changes to gitea (#525)
- log published version (#514)

## [0.4.15](https://github.com/MarcoIeni/release-plz/compare/release_plz_core-v0.4.14...release_plz_core-v0.4.15) - 2023-01-31

### Fixed
- *(release)* trust github workspace dir (#512)

## [0.4.14](https://github.com/MarcoIeni/release-plz/compare/release_plz_core-v0.4.13...release_plz_core-v0.4.14) - 2023-01-31

### Fixed
- handle new crate correctly (#509, #511)

### Other
- improve log (#502)

## [0.4.13](https://github.com/MarcoIeni/release-plz/compare/release_plz_core-v0.4.12...release_plz_core-v0.4.13) - 2023-01-27

### Fixed
- fix edit pr when a new file is present (#498)

### Other
- improve logging (#500)

## [0.4.12](https://github.com/MarcoIeni/release-plz/compare/release_plz_core-v0.4.11...release_plz_core-v0.4.12) - 2023-01-26

### Added
- edit GitHub release pr instead of closing it (#470)

### Other
- fix cargo clippy (#489)

## [0.4.11](https://github.com/MarcoIeni/release-plz/compare/release_plz_core-v0.4.10...release_plz_core-v0.4.11) - 2023-01-22

### Other
- update cargo (#473)

## [0.4.10](https://github.com/MarcoIeni/release-plz/compare/release_plz_core-v0.4.9...release_plz_core-v0.4.10) - 2023-01-17

### Other
- remove `octocrab` dependency (#467)

## [0.4.9](https://github.com/MarcoIeni/release-plz/compare/release_plz_core-v0.4.8...release_plz_core-v0.4.9) - 2023-01-16

### Added
- *(release-pr)* do not include the crate name if no workspace (#461)

### Fixed
- wrong log line (#464)

### Other
- fix typo in code (#463)

## [0.4.8](https://github.com/MarcoIeni/release-plz/compare/release_plz_core-v0.4.7...release_plz_core-v0.4.8) - 2023-01-16

### Fixed
- do not update changelog if new version exists (#452)
- changelog: fix link to first change (#450)

## [0.4.7](https://github.com/MarcoIeni/release-plz/compare/release_plz_core-v0.4.6...release_plz_core-v0.4.7) - 2023-01-15

### Added
- do not prefix crate name in tag for single crate projects (#444)

## [0.4.6](https://github.com/MarcoIeni/release-plz/compare/release_plz_core-v0.4.5...release_plz_core-v0.4.6) - 2023-01-12

### Added
- Include previous version in Pr Body (#430)

## [0.4.5](https://github.com/MarcoIeni/release-plz/compare/release_plz_core-v0.4.4...release_plz_core-v0.4.5) - 2023-01-11

### Other
- don't remove build metadata (#433)
- handle pre-releases (#425)

## [0.4.4](https://github.com/MarcoIeni/release-plz/compare/release_plz_core-v0.4.3...release_plz_core-v0.4.4) - 2023-01-07

### Added
- add body to git release (#411)

### Fixed
- *(release)* git-token is optional (#413)

## [0.4.3](https://github.com/MarcoIeni/release-plz/compare/release_plz_core-v0.4.2...release_plz_core-v0.4.3) - 2023-01-06

### Added
- Initial experimental support for GitHub releases.

### Other
- print error kind when copying directories (#408)
- use secret strings for tokens (#403)

## [0.4.2](https://github.com/MarcoIeni/release-plz/compare/release_plz_core-v0.4.1...release_plz_core-v0.4.2) - 2022-12-26

### Other
- fix additional clippy lints (#379)

## [0.4.1](https://github.com/MarcoIeni/release-plz/compare/release_plz_core-v0.4.0...release_plz_core-v0.4.1) - 2022-12-16

### Other
- remove `cargo-edit` dependency (#375)
- Add support for Gitea repositories (#372)

## [0.4.0] - 2022-12-12

### Added
- [**breaking**] Changelog: add GitHub release link to show the commits since the previous version (#371)

## [0.3.1] - 2022-11-17

### Changed
- don't release if the tag exists (#342)

## [0.3.0] - 2022-11-12

### Fixed
- [**breaking**] remove deprecated chrono::Date (#340)

## [0.2.21] - 2022-11-04

### Fixed
- clippy lints (#332)

### Other
- use workspace dependencies (#333)

## [0.2.20] - 2022-11-03

### Fixed
- *(typo)* update code comment for `Diff` struct field (#329)

## [0.2.19] - 2022-10-24

### Fixed
- include function removed from cargo-edit (#305)

### Other
- bump dependencies
- use cargo-clone-core library (#302)
- fix clippy lint (#297)

## [0.2.18] - 2022-07-16

### Fixed
- support symlinks (#213)

### Other
- update git cliff to version 8 (#212)
- update to cargo edit 10 (#211)
- *(deps)* bump serde_json from 1.0.81 to 1.0.82 (#202)
- remove unused lifetime (#206)

## [0.2.17] - 2022-06-18

### Other
- update clap and fake libraries (#186)
- *(deps)* bump tracing from 0.1.34 to 0.1.35 (#179)
- *(deps)* bump tokio from 1.19.1 to 1.19.2 (#178)
- *(deps)* bump tokio from 1.18.2 to 1.19.1 (#175)

## [0.2.16] - 2022-05-29

### Added
- add --allow-dirty flag to update command (#169)

## [0.2.15] - 2022-05-28

### Other
- skip pr field in instrument (#165)

## [0.2.14] - 2022-05-28

### Fixed
- remove link parsers (#155)

### Other
- *(deps)* bump crates-index from 0.18.7 to 0.18.8 (#144)
- *(deps)* bump expect-test from 1.2.2 to 1.3.0 (#141)

## [0.2.13] - 2022-05-26

### Added
- improve PR body (#139)

## [0.2.12] - 2022-05-19

### Other
- upgrade dependencies (#133)

## [0.2.11] - 2022-05-14

### Other
- updated the following local packages: git_cmd

## [0.2.10] - 2022-05-13

### Added
- read custom git cliff config (#126)

## [0.2.9] - 2022-05-10

### Added
- add ability to update all the dependencies in the lockfile (#123)

## [0.2.8] - 2022-05-08

### Added
- update package if one of its local dependencies is updated (#112)

## [0.2.7] - 2022-05-02

### Fixed
- *(release)* do not discard error (#105)

## [0.2.6] - 2022-05-01

### Fixed
- fix cargo update (#99)

### Changed
- update local dependencies versions (#102)
- remove default features from some deps (#101)

## [0.2.5] - 2022-04-27

### Added
- add `release` command (#89)
- Publish tag (#92)

### Fixed
- fix package equality

### Other
- update dependencies (#91)

## [0.2.4] - 2022-04-23

### Added
- *(release-pr)* close old release-plz prs when running release-plz (#81)
- update Cargo.lock, too (#78)

## [0.2.3] - 2022-04-11

### Fixed
- generate changelog for new packages (#72)
- add support for unpublished packages (#71)

### Changed
- name new branch as timestamp (#70)

## [0.2.2] - 2022-04-10

### Fixed
- remove all unwraps that are not in tests (#49)

### Other
- write package name and version in PR name (#58)
- improve workspace members error (#56)

## [0.2.1] - 2022-03-30

### Fixed
- fix changelog for release-pr (#39)
- remove changelog unwrap (#37)

### Added
- support alternative registries (#34)

### Other
- update crate description

## [0.2.0] - 2022-03-27

### Added
- [**breaking**] generate changelog with git-cliff (#29)

### Fixed
- fix breaking change message
- fix repository link
