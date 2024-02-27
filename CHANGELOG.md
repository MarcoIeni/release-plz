# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.3.49](https://github.com/MarcoIeni/release-plz/compare/release-plz-v0.3.48...release-plz-v0.3.49) - 2024-02-27

### Other
- error if committed Cargo.lock is in `.gitignore` ([#1294](https://github.com/MarcoIeni/release-plz/pull/1294))

## [0.3.48](https://github.com/MarcoIeni/release-plz/compare/release-plz-v0.3.47...release-plz-v0.3.48) - 2024-02-25

### Other
- update Cargo.lock dependencies
- don't log big release request ([#1300](https://github.com/MarcoIeni/release-plz/pull/1300))

## [0.3.47](https://github.com/MarcoIeni/release-plz/compare/release-plz-v0.3.46...release-plz-v0.3.47) - 2024-02-25

### Added
- prepare release if commits respect the `release_commits` regex ([#1278](https://github.com/MarcoIeni/release-plz/pull/1278))

### Other
- update cargo to v0.77 ([#1296](https://github.com/MarcoIeni/release-plz/pull/1296))

## [0.3.46](https://github.com/MarcoIeni/release-plz/compare/release-plz-v0.3.45...release-plz-v0.3.46) - 2024-02-23

### Added
- add `all-static` feature ([#1287](https://github.com/MarcoIeni/release-plz/pull/1287))

### Fixed
- allow configuring the `release` flag ([#1290](https://github.com/MarcoIeni/release-plz/pull/1290))

### Other
- enhance test `schema_is_up_to_date` ([#1285](https://github.com/MarcoIeni/release-plz/pull/1285))

## [0.3.45](https://github.com/MarcoIeni/release-plz/compare/release-plz-v0.3.44...release-plz-v0.3.45) - 2024-02-11

### Added
- allow customizing git release name with tera template. [#677](https://github.com/MarcoIeni/release-plz/pull/677) ([#1260](https://github.com/MarcoIeni/release-plz/pull/1260))

### Fixed
- *(config)* deny unknown fields ([#1263](https://github.com/MarcoIeni/release-plz/pull/1263))

## [0.3.44](https://github.com/MarcoIeni/release-plz/compare/release-plz-v0.3.43...release-plz-v0.3.44) - 2024-02-09

### Added
- allow customizing git tag name with tera template ([#1256](https://github.com/MarcoIeni/release-plz/pull/1256))

## [0.3.43](https://github.com/MarcoIeni/release-plz/compare/release-plz-v0.3.42...release-plz-v0.3.43) - 2024-02-06

### Added
- add changelog config in `release-plz.toml` ([#1198](https://github.com/MarcoIeni/release-plz/pull/1198))

### Fixed
- update local dependencies specified in the workspace manifest ([#1251](https://github.com/MarcoIeni/release-plz/pull/1251))
- check cargo token only when publishing ([#1250](https://github.com/MarcoIeni/release-plz/pull/1250))

### Other
- fix tests on mac ([#1242](https://github.com/MarcoIeni/release-plz/pull/1242))

## [0.3.42](https://github.com/MarcoIeni/release-plz/compare/release-plz-v0.3.41...release-plz-v0.3.42) - 2024-01-26

### Added
- add `git_release_type` configuration option to allow GitHub/Gitea prereleases ([#1228](https://github.com/MarcoIeni/release-plz/pull/1228))

### Fixed
- support rust-toolchain.toml file ([#1234](https://github.com/MarcoIeni/release-plz/pull/1234))

### Other
- add context to some errors ([#1232](https://github.com/MarcoIeni/release-plz/pull/1232))

## [0.3.41](https://github.com/MarcoIeni/release-plz/compare/release-plz-v0.3.40...release-plz-v0.3.41) - 2024-01-23

### Added
- use github graphql api for commits to have the GitHub "Verified" badge on release-plz commits
  ([#1201](https://github.com/MarcoIeni/release-plz/pull/1201))

### Other
- update Cargo.lock dependencies

## [0.3.40](https://github.com/MarcoIeni/release-plz/compare/release-plz-v0.3.39...release-plz-v0.3.40) - 2024-01-20

### Fixed
- Correct dependency update behavior for release-pr ([#1217](https://github.com/MarcoIeni/release-plz/pull/1217))

### Other
- update dependencies ([#1213](https://github.com/MarcoIeni/release-plz/pull/1213))

## [0.3.39](https://github.com/MarcoIeni/release-plz/compare/release-plz-v0.3.38...release-plz-v0.3.39) - 2024-01-16

### Added
- Add release flag ([#1098](https://github.com/MarcoIeni/release-plz/pull/1098))

### Fixed
- Prevent error if Cargo.lock doesn't exist during portions of commit history ([#1205](https://github.com/MarcoIeni/release-plz/pull/1205))

### Other
- improve public packages error message ([#1187](https://github.com/MarcoIeni/release-plz/pull/1187))
- add debug statement when Cargo.toml differs ([#1184](https://github.com/MarcoIeni/release-plz/pull/1184))
- less verbose logs ([#1183](https://github.com/MarcoIeni/release-plz/pull/1183))

## [0.3.38](https://github.com/MarcoIeni/release-plz/compare/release-plz-v0.3.37...release-plz-v0.3.38) - 2023-12-30

### Other
- flatten part of config struct ([#1154](https://github.com/MarcoIeni/release-plz/pull/1154))
- remove unuseful function ([#1166](https://github.com/MarcoIeni/release-plz/pull/1166))
- simplify code ([#1165](https://github.com/MarcoIeni/release-plz/pull/1165))

## [0.3.37](https://github.com/MarcoIeni/release-plz/compare/release-plz-v0.3.36...release-plz-v0.3.37) - 2023-12-19

### Fixed
- restore changes introduced by `cargo package` ([#1152](https://github.com/MarcoIeni/release-plz/pull/1152))

## [0.3.36](https://github.com/MarcoIeni/release-plz/compare/release-plz-v0.3.35...release-plz-v0.3.36) - 2023-12-17

### Other
- update dependencies

## [0.3.35](https://github.com/MarcoIeni/release-plz/compare/release-plz-v0.3.34...release-plz-v0.3.35) - 2023-12-16

### Added
- return error if tag exists and package isn't published ([#1143](https://github.com/MarcoIeni/release-plz/pull/1143))
- support packages with git dependencies ([#1141](https://github.com/MarcoIeni/release-plz/pull/1141))
- distinguish dependency update type ([#1140](https://github.com/MarcoIeni/release-plz/pull/1140))

### Fixed
- internal dependency conflict ([#1135](https://github.com/MarcoIeni/release-plz/pull/1135))

### Other
- update dependencies
- *(deps)* bump cargo to 0.75.1 ([#1137](https://github.com/MarcoIeni/release-plz/pull/1137))

## [0.3.34](https://github.com/MarcoIeni/release-plz/compare/release-plz-v0.3.33...release-plz-v0.3.34) - 2023-12-13

### Fixed
- pass previous version to git-cliff ([#1134](https://github.com/MarcoIeni/release-plz/pull/1134))

### Other
- update dependencies

## [0.3.33](https://github.com/MarcoIeni/release-plz/compare/release-plz-v0.3.32...release-plz-v0.3.33) - 2023-12-08

### Added
- new generate-schema command to generate a JSON schema for the configuration ([#1101](https://github.com/MarcoIeni/release-plz/pull/1101))

### Other
- *(args)* hide the environment value of git token ([#1124](https://github.com/MarcoIeni/release-plz/pull/1124))
- update git-cliff references ([#1115](https://github.com/MarcoIeni/release-plz/pull/1115))

## [0.3.32](https://github.com/MarcoIeni/release-plz/compare/release-plz-v0.3.31...release-plz-v0.3.32) - 2023-12-04

### Fixed
- support projects with external readme ([#1110](https://github.com/MarcoIeni/release-plz/pull/1110))
- pass full commit message to git-cliff ([#1103](https://github.com/MarcoIeni/release-plz/pull/1103)) ([#1104](https://github.com/MarcoIeni/release-plz/pull/1104))

### Other
- update dependencies

## [0.3.31](https://github.com/MarcoIeni/release-plz/compare/release-plz-v0.3.30...release-plz-v0.3.31) - 2023-11-30

### Added
- add publish_timeout to avoid release blocking issue, fix [#1015](https://github.com/MarcoIeni/release-plz/pull/1015) ([#1088](https://github.com/MarcoIeni/release-plz/pull/1088))
- prevent typos in overrides ([#1080](https://github.com/MarcoIeni/release-plz/pull/1080))
- Update a package only if edited file belongs to `cargo package --list` ([#1089](https://github.com/MarcoIeni/release-plz/pull/1089))

### Fixed
- resolve issue on Windows machines that use CRLF that would duplicate the header on each update ([#1083](https://github.com/MarcoIeni/release-plz/pull/1083))

### Other
- document Gitea releases ([#1076](https://github.com/MarcoIeni/release-plz/pull/1076))

## [0.3.30](https://github.com/MarcoIeni/release-plz/compare/release-plz-v0.3.29...release-plz-v0.3.30) - 2023-11-08

### Added
- add `pr_draft` config option ([#1061](https://github.com/MarcoIeni/release-plz/pull/1061))
- support .release-plz.toml as a config file ([#1057](https://github.com/MarcoIeni/release-plz/pull/1057))

## [0.3.29](https://github.com/MarcoIeni/release-plz/compare/release-plz-v0.3.28...release-plz-v0.3.29) - 2023-10-27

### Fixed
- use `release-plz-` rather than `release-plz/` branch prefix ([#1041](https://github.com/MarcoIeni/release-plz/pull/1041))
- use registry argument on publish ([#1050](https://github.com/MarcoIeni/release-plz/pull/1050))

## [0.3.28](https://github.com/MarcoIeni/release-plz/compare/release-plz-v0.3.27...release-plz-v0.3.28) - 2023-10-15

### Added
- *(args)* support `GIT_TOKEN` variable ([#1008](https://github.com/MarcoIeni/release-plz/pull/1008)) ([#1026](https://github.com/MarcoIeni/release-plz/pull/1026))

### Fixed
- ignore `.ignore` files ([#1036](https://github.com/MarcoIeni/release-plz/pull/1036))

## [0.3.27](https://github.com/MarcoIeni/release-plz/compare/release-plz-v0.3.26...release-plz-v0.3.27) - 2023-09-30

### Other
- update dependencies

## [0.3.26](https://github.com/MarcoIeni/release-plz/compare/release-plz-v0.3.25...release-plz-v0.3.26) - 2023-09-30

### Added
- avoid copying gitignored files ([#1000](https://github.com/MarcoIeni/release-plz/pull/1000)) ([#1001](https://github.com/MarcoIeni/release-plz/pull/1001))

### Fixed
- parse changelog header correctly ([#1007](https://github.com/MarcoIeni/release-plz/pull/1007))

### Other
- update dependencies

## [0.3.25](https://github.com/MarcoIeni/release-plz/compare/release-plz-v0.3.24...release-plz-v0.3.25) - 2023-09-24

### Added
- add ability to create draft git release ([#986](https://github.com/MarcoIeni/release-plz/pull/986))

### Fixed
- respect git-cliff sort order ([#985](https://github.com/MarcoIeni/release-plz/pull/985))

## [0.3.24](https://github.com/MarcoIeni/release-plz/compare/release-plz-v0.3.23...release-plz-v0.3.24) - 2023-09-17

### Fixed
- don't discard error context ([#971](https://github.com/MarcoIeni/release-plz/pull/971))
- don't publish examples ([#974](https://github.com/MarcoIeni/release-plz/pull/974))

## [0.3.23](https://github.com/MarcoIeni/release-plz/compare/release-plz-v0.3.22...release-plz-v0.3.23) - 2023-09-16

### Added
- allow disabling git tag ([#968](https://github.com/MarcoIeni/release-plz/pull/968))
- pass commit ids to git-cliff ([#967](https://github.com/MarcoIeni/release-plz/pull/967))

### Other
- add additional clippy lints ([#965](https://github.com/MarcoIeni/release-plz/pull/965))

## [0.3.22](https://github.com/MarcoIeni/release-plz/compare/release-plz-v0.3.21...release-plz-v0.3.22) - 2023-09-11

### Added
- *(release-pr)* sign release-plz commit ([#956](https://github.com/MarcoIeni/release-plz/pull/956))

## [0.3.21](https://github.com/MarcoIeni/release-plz/compare/release-plz-v0.3.20...release-plz-v0.3.21) - 2023-09-08

### Other
- update dependencies
- *(ci)* check links ([#941](https://github.com/MarcoIeni/release-plz/pull/941))
- fix clippy lint ([#931](https://github.com/MarcoIeni/release-plz/pull/931))

## [0.3.20](https://github.com/MarcoIeni/release-plz/compare/release-plz-v0.3.19...release-plz-v0.3.20) - 2023-08-22

### Fixed
- allow specifying config file path ([#924](https://github.com/MarcoIeni/release-plz/pull/924))

### Other
- test release-plz release ([#892](https://github.com/MarcoIeni/release-plz/pull/892))
- move release-plz changelog ([#917](https://github.com/MarcoIeni/release-plz/pull/917))
- add feature flag to ignore docker tests ([#914](https://github.com/MarcoIeni/release-plz/pull/914))
- static openssl ([#920](https://github.com/MarcoIeni/release-plz/pull/920))
- improve http error messages ([#921](https://github.com/MarcoIeni/release-plz/pull/921))
- update git-cliff ([#919](https://github.com/MarcoIeni/release-plz/pull/919))

## [0.3.19](https://github.com/MarcoIeni/release-plz/compare/release-plz-v0.3.18...release-plz-v0.3.19) - 2023-08-16

### Fixed
- release in https git repos ([#912](https://github.com/MarcoIeni/release-plz/pull/912))

## [0.3.18](https://github.com/MarcoIeni/release-plz/compare/release-plz-v0.3.17...release-plz-v0.3.18) - 2023-08-14

### Added
- add `changelog_include` option ([#904](https://github.com/MarcoIeni/release-plz/pull/904))

### Other
- add tests for gitea ([#421](https://github.com/MarcoIeni/release-plz/pull/421))

## [0.3.17](https://github.com/MarcoIeni/release-plz/compare/release-plz-v0.3.16...release-plz-v0.3.17) - 2023-08-02

### Fixed
- update workspace version in dependencies ([#889](https://github.com/MarcoIeni/release-plz/pull/889))

## [0.3.16](https://github.com/MarcoIeni/release-plz/compare/release-plz-v0.3.15...release-plz-v0.3.16) - 2023-07-25

### Added
- *(release)* add support for sparse registry URLs ([#863](https://github.com/MarcoIeni/release-plz/pull/863))

## [0.3.15](https://github.com/MarcoIeni/release-plz/compare/release-plz-v0.3.14...release-plz-v0.3.15) - 2023-06-26

### Fixed
- copy symlinks ([#827](https://github.com/MarcoIeni/release-plz/pull/827))

## [0.3.14](https://github.com/MarcoIeni/release-plz/compare/release-plz-v0.3.13...release-plz-v0.3.14) - 2023-06-10

### Fixed
- ignore Cargo.lock dev dependencies changes ([#820](https://github.com/MarcoIeni/release-plz/pull/820))

## [0.3.13](https://github.com/MarcoIeni/release-plz/compare/release-plz-v0.3.12...release-plz-v0.3.13) - 2023-06-09

### Fixed
- update changelog correctly when workspace version specified ([#816](https://github.com/MarcoIeni/release-plz/pull/816))

## [0.3.12](https://github.com/MarcoIeni/release-plz/compare/release-plz-v0.3.11...release-plz-v0.3.12) - 2023-06-09

- update dependencies ([#814](https://github.com/MarcoIeni/release-plz/pull/814))
- stop looking at git history if commit tagged ([#813](https://github.com/MarcoIeni/release-plz/pull/813))

## [0.3.11](https://github.com/MarcoIeni/release-plz/compare/release-plz-v0.3.10...release-plz-v0.3.11) - 2023-05-31

### Fixed
- *(workspaces)* report correct version update ([#802](https://github.com/MarcoIeni/release-plz/pull/802))

## [0.3.10](https://github.com/MarcoIeni/release-plz/compare/release-plz-v0.3.9...release-plz-v0.3.10) - 2023-05-24

### Added
- add pr/issue link to changelog entries (#793)

### Other
- parse cargo lock faster (#795)

## [0.3.9](https://github.com/MarcoIeni/release-plz/compare/release-plz-v0.3.8...release-plz-v0.3.9) - 2023-05-21

### Added
- check if `Cargo.lock` packages were updated (#784)

### Fixed
- support nested crates (#783)

## [0.3.8](https://github.com/MarcoIeni/release-plz/compare/release-plz-v0.3.7...release-plz-v0.3.8) - 2023-05-08

### Other
- run cargo-semver-check in parallel (#766)
- represent semver_check config as bool (#765)

## [0.3.7](https://github.com/MarcoIeni/release-plz/compare/release-plz-v0.3.6...release-plz-v0.3.7) - 2023-05-07

### Other
- Performance improvement: run semver-checks only on changed packages (#754)

## [0.3.6](https://github.com/MarcoIeni/release-plz/compare/release-plz-v0.3.5...release-plz-v0.3.6) - 2023-05-07

### Fixed
- abort failed rebase (#760)

## [0.3.5](https://github.com/MarcoIeni/release-plz/compare/release-plz-v0.3.4...release-plz-v0.3.5) - 2023-05-05

### Fixed
- set repo url also for release command (#751)

### Added
- Add `publish` config option to disable publishing to the cargo registry (#718)

## [0.3.4](https://github.com/MarcoIeni/release-plz/compare/release-plz-v0.3.3...release-plz-v0.3.4) - 2023-04-27

### Fixed
- don't compare ignored files (#739)

## [0.3.3](https://github.com/MarcoIeni/release-plz/compare/release-plz-v0.3.2...release-plz-v0.3.3) - 2023-04-25

### Fixed
- downgrade cargo to fix windows compilation

## [0.3.2](https://github.com/MarcoIeni/release-plz/compare/release-plz-v0.3.1...release-plz-v0.3.2) - 2023-04-24

### Other
- only add commit title in changelog (#729)

## [0.3.1](https://github.com/MarcoIeni/release-plz/compare/release-plz-v0.3.0...release-plz-v0.3.1) - 2023-04-21

### Added
- add `pr_labels` configuration option to add labels to the PR opened by release-plz (#708)

## [0.3.0](https://github.com/MarcoIeni/release-plz/compare/release-plz-v0.2.63...release-plz-v0.3.0) - 2023-04-16

### Added
- [**breaking**] changed config file format. See [docs](https://release-plz.ieni.dev/docs/config.html).
- [**breaking**] removed `--git-release` flag. Now git releases are enabled by default.
  You can disable them with the `git_release_enable` configuration option.
- make cargo publish flags configurable (#684)

### Fixed
- config package override (#695)
- don't return early when publishing crates (#691)

## [0.2.63](https://github.com/MarcoIeni/release-plz/compare/release-plz-v0.2.62...release-plz-v0.2.63) - 2023-04-05

### Fixed
- changelog path handling (#669)
- detect allow-dirty error (#666)

## [0.2.62](https://github.com/MarcoIeni/release-plz/compare/release-plz-v0.2.61...release-plz-v0.2.62) - 2023-04-02

### Added
- allow to provide a custom changelog path (#653)

## [0.2.61](https://github.com/MarcoIeni/release-plz/compare/release-plz-v0.2.60...release-plz-v0.2.61) - 2023-04-02

### Other
- detect custom changelog header (#651)

## [0.2.60](https://github.com/MarcoIeni/release-plz/compare/release-plz-v0.2.59...release-plz-v0.2.60) - 2023-04-02

### Other
- read opened PRs with empty body (#649)

## [0.2.59](https://github.com/MarcoIeni/release-plz/compare/release-plz-v0.2.58...release-plz-v0.2.59) - 2023-04-01

### Added
- Add config file. See the [docs](https://release-plz.ieni.dev/docs/config.html) (#634)

## [0.2.58](https://github.com/MarcoIeni/release-plz/compare/release-plz-v0.2.57...release-plz-v0.2.58) - 2023-03-27

### Added
- add release-plz config file (#589). Experimental, not documented yet.

## [0.2.57](https://github.com/MarcoIeni/release-plz/compare/release-plz-v0.2.56...release-plz-v0.2.57) - 2023-03-19

### Added
- *(release)* add GitLab support (#591).
  `release-plz release-pr` GitLab support is still missing.

## [0.2.56](https://github.com/MarcoIeni/release-plz/compare/release-plz-v0.2.55...release-plz-v0.2.56) - 2023-03-17

### Fixed
- update pr: do git fetch before rebase (#607)

## [0.2.55](https://github.com/MarcoIeni/release-plz/compare/release-plz-v0.2.54...release-plz-v0.2.55) - 2023-03-13

### Added
- write changelog in pr body (#598)

## [0.2.54](https://github.com/MarcoIeni/release-plz/compare/release-plz-v0.2.53...release-plz-v0.2.54) - 2023-03-10

### Fix
- update to cargo v0.69 to be compatible with sparse index.

## [0.2.53](https://github.com/MarcoIeni/release-plz/compare/release-plz-v0.2.52...release-plz-v0.2.53) - 2023-03-09

### Added
- include version in pr title for single crate (#593)

### Other
- retry failing http calls (#585)

## [0.2.52](https://github.com/MarcoIeni/release-plz/compare/release-plz-v0.2.51...release-plz-v0.2.52) - 2023-03-04

### Added
- detect circular dependency (#581)

## [0.2.51](https://github.com/MarcoIeni/release-plz/compare/release-plz-v0.2.50...release-plz-v0.2.51) - 2023-02-27

### Fix
- remove ansi escape sequences in cargo-semver-checks output (#575)

## [0.2.50](https://github.com/MarcoIeni/release-plz/compare/release-plz-v0.2.49...release-plz-v0.2.50) - 2023-02-26

### Added
- Add cargo-semver-checks integration. If the `cargo-semver-checks` binary is present, release-plz uses
  it to check semver compatibility. If `cargo-semver-checks` detects an API breaking change, release-plz
  updates the major version. (#568)

### Fixed
- when editing a release-pr, update pr title and body (#571)

## [0.2.49](https://github.com/MarcoIeni/release-plz/compare/release-plz-v0.2.48...release-plz-v0.2.49) - 2023-02-20

### Other
- update dependencies
- remove unused check (#559)

## [0.2.48](https://github.com/MarcoIeni/release-plz/compare/release-plz-v0.2.47...release-plz-v0.2.48) - 2023-02-18

### Fixed
- *(release)* fix github release (#556)

## [0.2.47](https://github.com/MarcoIeni/release-plz/compare/release-plz-v0.2.46...release-plz-v0.2.47) - 2023-02-18

### Fixed
- *(release)* trust gh workspace (#553)

## [0.2.46](https://github.com/MarcoIeni/release-plz/compare/release-plz-v0.2.45...release-plz-v0.2.46) - 2023-02-18

### Other
- `release-plz release` creates a release in Gitea, too (#539)

## [0.2.45](https://github.com/MarcoIeni/release-plz/compare/release-plz-v0.2.44...release-plz-v0.2.45) - 2023-02-11

### Added
- *(release)* add the possibility to add `--no-verify` and `--allow-dirty` as cargo publish flags (#532)

## [0.2.44](https://github.com/MarcoIeni/release-plz/compare/release-plz-v0.2.43...release-plz-v0.2.44) - 2023-02-10

### Added
- update pr in gitea (#530)

### Fixed
- update branch from main before updating PR (#528)

## [0.2.43](https://github.com/MarcoIeni/release-plz/compare/release-plz-v0.2.42...release-plz-v0.2.43) - 2023-02-08

### Added
- add changelog changes to gitea (#525)
- log published version (#514)

## [0.2.42](https://github.com/MarcoIeni/release-plz/compare/release-plz-v0.2.41...release-plz-v0.2.42) - 2023-01-31

### Other
- *(release)* trust github workspace dir (#512)

## [0.2.41](https://github.com/MarcoIeni/release-plz/compare/release-plz-v0.2.40...release-plz-v0.2.41) - 2023-01-31

### Fixed
- handle new crate correctly (#509, #511)

### Other
- improve log (#502)

## [0.2.40](https://github.com/MarcoIeni/release-plz/compare/release-plz-v0.2.39...release-plz-v0.2.40) - 2023-01-27

### Fixed
- fix edit pr when a new file is present (#498)

### Other
- improve logging (#500)

## [0.2.39](https://github.com/MarcoIeni/release-plz/compare/release-plz-v0.2.38...release-plz-v0.2.39) - 2023-01-26

### Added
- edit GitHub release pr instead of closing it (#470)

### Other
- fix cargo clippy (#489)

## [0.2.38](https://github.com/MarcoIeni/release-plz/compare/release-plz-v0.2.37...release-plz-v0.2.38) - 2023-01-22

### Added
- add new `check-updates` command to check if release-plz is up to date (#477) (#471)

## [0.2.37](https://github.com/MarcoIeni/release-plz/compare/release-plz-v0.2.36...release-plz-v0.2.37) - 2023-01-22

### Other
- update cargo (#473)

## [0.2.36](https://github.com/MarcoIeni/release-plz/compare/release-plz-v0.2.35...release-plz-v0.2.36) - 2023-01-17

### Other
- remove `octocrab` dependency (#467)

## [0.2.35](https://github.com/MarcoIeni/release-plz/compare/release-plz-v0.2.34...release-plz-v0.2.35) - 2023-01-16

### Added
- *(release-pr)* do not include the crate name if there is only one
  publishable package in the project (#461)

### Fixed
- wrong log line (#464)

## [0.2.34](https://github.com/MarcoIeni/release-plz/compare/release-plz-v0.2.33...release-plz-v0.2.34) - 2023-01-16

### Fixed
- do not update changelog if new version exists (#452)
- changelog: fix link to first change (#450)

### Other
- *(deps)* bump assert_cmd from 2.0.7 to 2.0.8 (#453)

## [0.2.33](https://github.com/MarcoIeni/release-plz/compare/release-plz-v0.2.32...release-plz-v0.2.33) - 2023-01-15

### Added
- do not prefix crate name in tag for single crate projects (#444)

## [0.2.32](https://github.com/MarcoIeni/release-plz/compare/release-plz-v0.2.31...release-plz-v0.2.32) - 2023-01-12

### Added
- Include previous version in Pr Body (#430)

## [0.2.31](https://github.com/MarcoIeni/release-plz/compare/release-plz-v0.2.30...release-plz-v0.2.31) - 2023-01-11

### Added
- don't remove build metadata (#433)
- handle pre-releases (#425)

## [0.2.30](https://github.com/MarcoIeni/release-plz/compare/release-plz-v0.2.29...release-plz-v0.2.30) - 2023-01-07

### Added
- add body to git release (#411)

### Fixed
- *(release)* git-token is optional (#413)

## [0.2.29](https://github.com/MarcoIeni/release-plz/compare/release-plz-v0.2.28...release-plz-v0.2.29) - 2023-01-06

### Added
- Initial support for GitHub releases. You can create a GitHub release when running `release-plz release` by using the `--git-release` flag.

### Other
- print error kind when copying directories (#408)
- make errors more visible (#405)
- use secret strings for tokens (#403)

## [0.2.28](https://github.com/MarcoIeni/release-plz/compare/release-plz-v0.2.27...release-plz-v0.2.28) - 2022-12-26

### Fixed
- reintroduce github-token flag (#389)

## [0.2.27](https://github.com/MarcoIeni/release-plz/compare/release-plz-v0.2.26...release-plz-v0.2.27) - 2022-12-16

### Other
- Add support for Gitea repositories (#372)

## [0.2.26](https://github.com/MarcoIeni/release-plz/compare/release-plz-v0.2.25...release-plz-v0.2.26) - 2022-12-12

### Added
- Changelog: add GitHub release link to show the commits since the previous version (#371)

### Other
- *(deps)* bump assert_cmd from 2.0.6 to 2.0.7 (#366)
- *(completions)* add tests for shell completions (#330) (#349)

## [0.2.25](https://github.com/MarcoIeni/release-plz/compare/release-plz-v0.2.24...release-plz-v0.2.25) - 2022-11-17

### Changed
- don't release if the tag exists (#342)

## [0.2.24](https://github.com/MarcoIeni/release-plz/compare/release-plz-v0.2.23...release-plz-v0.2.24) - 2022-11-12

### Fixed
- breaking remove deprecated chrono::Date (#340)

## [0.2.23](https://github.com/MarcoIeni/release-plz/compare/release-plz-v0.2.22...release-plz-v0.2.23) - 2022-11-04

### Fixed
- github token parsing (#334)

### Other
- use workspace dependencies (#333)

## [0.2.22](https://github.com/MarcoIeni/release-plz/compare/release-plz-v0.2.21...release-plz-v0.2.22) - 2022-11-03

### Fixed
- *(args)* use the correct case for conflicting arguments (#328)

## [0.2.21](https://github.com/MarcoIeni/release-plz/compare/release-plz-v0.2.20...release-plz-v0.2.21) - 2022-11-01

### Other
- update to clap v4 (#325)

## [0.2.20](https://github.com/MarcoIeni/release-plz/compare/release-plz-v0.2.19...release-plz-v0.2.20) - 2022-10-24

### Other
- bump dependencies

## [0.2.19](https://github.com/MarcoIeni/release-plz/compare/release-plz-v0.2.18...release-plz-v0.2.19) - 2022-07-16

### Other
- update git cliff to version 8 (#212)
- *(deps)* bump clap_complete from 3.2.2 to 3.2.3 (#201)
- *(deps)* bump clap from 3.2.6 to 3.2.8 (#200)
- *(deps)* bump tracing-subscriber from 0.3.11 to 0.3.14 (#199)

## [0.2.18](https://github.com/MarcoIeni/release-plz/compare/release-plz-v0.2.17...release-plz-v0.2.18) - 2022-06-18

### Added
- add `generate-completions` command to generate shell completions file. (#177)

### Other
- *(deps)* bump clap and fake libraries (#186)
- *(deps)* bump tracing from 0.1.34 to 0.1.35 (#179)
- *(deps)* bump tokio from 1.19.1 to 1.19.2 (#178)
- *(deps)* bump tokio from 1.18.2 to 1.19.1 (#175)
- *(deps)* bump git-url-parse from 0.4.1 to 0.4.2 (#172)

## [0.2.17](https://github.com/MarcoIeni/release-plz/compare/release-plz-v0.2.16...release-plz-v0.2.17) - 2022-05-29

### Added
- add --allow-dirty flag to update command (#169)

## [0.2.16](https://github.com/MarcoIeni/release-plz/compare/release-plz-v0.2.15...release-plz-v0.2.16) - 2022-05-29

### Added
- add `verbose` flag (#167)

## [0.2.15](https://github.com/MarcoIeni/release-plz/compare/release-plz-v0.2.14...release-plz-v0.2.15) - 2022-05-28

### Other
- skip pr field in logs (#165)

## [0.2.14](https://github.com/MarcoIeni/release-plz/compare/release-plz-v0.2.13...release-plz-v0.2.14) - 2022-05-28

### Other
- update dependencies (#160)

## [0.2.13](https://github.com/MarcoIeni/release-plz/compare/release-plz-v0.2.12...release-plz-v0.2.13) - 2022-05-28

### Other
- updated the following local packages: release_plz_core

## [0.2.12](https://github.com/MarcoIeni/release-plz/compare/release-plz-v0.2.11...release-plz-v0.2.12) - 2022-05-26

### Other
- improve PR body (#139)

## [0.2.11](https://github.com/MarcoIeni/release-plz/compare/release-plz-v0.2.10...release-plz-v0.2.11) - 2022-05-19

### Other
- upgrade dependencies (#133)

## [0.2.10](https://github.com/MarcoIeni/release-plz/compare/release-plz-v0.2.9...release-plz-v0.2.10) - 2022-05-14

### Added
- infer repo url (#128)

## [0.2.9](https://github.com/MarcoIeni/release-plz/compare/release-plz-v0.2.8...release-plz-v0.2.9) - 2022-05-13

### Added
- read custom git cliff config (#126)

## [0.2.8](https://github.com/MarcoIeni/release-plz/compare/release-plz-v0.2.7...release-plz-v0.2.8) - 2022-05-10

### Added
- add ability to update all the dependencies in the lockfile with the `-u` cli option (#123)

## [0.2.7](https://github.com/MarcoIeni/release-plz/compare/release-plz-v0.2.6...release-plz-v0.2.7) - 2022-05-08

### Other
- update package if one of its local dependencies is updated (#112)

## [0.2.6](https://github.com/MarcoIeni/release-plz/compare/release-plz-v0.2.5...release-plz-v0.2.6) - 2022-05-02

### Changed
- update `release_plz_core` to 0.2.7

## [0.2.5](https://github.com/MarcoIeni/release-plz/compare/release-plz-v0.2.4...release-plz-v0.2.5) - 2022-05-01

### Changed
- update `release_plz_core` to 0.2.6

## [0.2.4](https://github.com/MarcoIeni/release-plz/compare/release-plz-v0.2.3...release-plz-v0.2.4) - 2022-04-27

### Added
- add `release` command (#89)
- *(cli)* forbid empty values in args (#88)

### Other
- *(args)* refactor (#87)

## [0.2.3](https://github.com/MarcoIeni/release-plz/compare/release-plz-v0.2.2...release-plz-v0.2.3) - 2022-04-23

### Added
- *(release-pr)* close old release-plz prs when running release-plz (#81)
- update Cargo.lock, too (#78)

## [0.2.2](https://github.com/MarcoIeni/release-plz/releases/tag/release-plz-v0.2.2) - 2022-04-10

### Fixed
- remove all unwraps that are not in tests (#49)

## [0.2.1] - 2022-03-30

### Added
- support alternative registries (#34)

### Other
- update crate description

## [0.2.0] - 2022-03-27

### Added
- [**breaking**] generate changelog with git-cliff (#29)
