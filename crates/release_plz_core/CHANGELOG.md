# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
