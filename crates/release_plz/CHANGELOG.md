# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.17] - 2022-05-29

### Added
- add --allow-dirty flag to update command (#169)

## [0.2.16] - 2022-05-29

### Added
- add `verbose` flag (#167)

## [0.2.15] - 2022-05-28

### Other
- skip pr field in logs (#165)

## [0.2.14] - 2022-05-28

### Other
- update dependencies (#160)

## [0.2.13] - 2022-05-28

### Other
- updated the following local packages: release_plz_core

## [0.2.12] - 2022-05-26

### Other
- improve PR body (#139)

## [0.2.11] - 2022-05-19

### Other
- upgrade dependencies (#133)

## [0.2.10] - 2022-05-14

### Added
- infer repo url (#128)

## [0.2.9] - 2022-05-13

### Added
- read custom git cliff config (#126)

## [0.2.8] - 2022-05-10

### Added
- add ability to update all the dependencies in the lockfile with the `-u` cli option (#123)

## [0.2.7] - 2022-05-08

### Other
- update package if one of its local dependencies is updated (#112)

## [0.2.6] - 2022-05-02

### Changed
- update `release_plz_core` to 0.2.7

## [0.2.5] - 2022-05-01

### Changed
- update `release_plz_core` to 0.2.6

## [0.2.4] - 2022-04-27

### Added
- add `release` command (#89)
- *(cli)* forbid empty values in args (#88)

### Other
- *(args)* refactor (#87)

## [0.2.3] - 2022-04-23

### Added
- *(release-pr)* close old release-plz prs when running release-plz (#81)
- update Cargo.lock, too (#78)

## [0.2.2] - 2022-04-10

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
