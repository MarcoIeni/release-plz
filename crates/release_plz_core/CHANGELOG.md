# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.19] - 2022-10-09

### Other
- use cargo-clone-core library (#302)
- *(deps)* bump cargo-edit from 0.11.1 to 0.11.3 (#295)
- fix clippy lint (#297)
- *(deps)* bump anyhow from 1.0.64 to 1.0.65 (#281)
- *(deps)* bump cargo-edit from 0.10.4 to 0.11.1 (#285)
- *(deps)* bump url from 2.2.2 to 2.3.0 (#277)
- *(deps)* bump anyhow from 1.0.62 to 1.0.64 (#272)
- *(deps)* bump serde_json from 1.0.83 to 1.0.85 (#265)
- update anyhow to 1.0.61 (#263)
- *(deps)* bump anyhow from 1.0.58 to 1.0.59 (#235)
- *(deps)* bump chrono from 0.4.20 to 0.4.22 (#252)
- *(deps)* bump crates-index from 0.18.8 to 0.18.9 (#250)
- *(deps)* bump octocrab from 0.16.0 to 0.17.0 (#249)
- *(deps)* bump chrono from 0.4.19 to 0.4.20 (#244)
- *(deps)* bump wiremock from 0.5.13 to 0.5.14 (#243)
- *(deps)* bump serde_json from 1.0.82 to 1.0.83 (#241)
- *(deps)* bump tracing from 0.1.35 to 0.1.36 (#234)
- *(deps)* bump cargo-edit from 0.10.2 to 0.10.4 (#232)
- *(deps)* bump expect-test from 1.3.0 to 1.4.0 (#227)
- *(deps)* bump cargo-edit from 0.10.1 to 0.10.2 (#225)

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
