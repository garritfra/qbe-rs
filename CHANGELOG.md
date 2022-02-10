# Changelog

All notable changes to this project will be documented in this file.

## Unreleased

### Added

- Data types now implement common traits (`Debug`, `Clone`, `Eq`, `PartialEq`,
`Ord`, `PartialOrd`, `Hash`, `Default` and `Copy` ) where applicable

### Changed

-   Remove `Qbe` prefix from data structures. `QbeValue` becomes `qbe::Value`

## [0.1.0] - 2022-02-09

### Added

-   Tests
-   Hello World example

### Changed

-   `QbeBlock` now has `statements` instead of `instructions`

## [0.0.1] - 2022-02-08

### Added

-   Initial release (taken over from the [Antimony](https://github.com/antimony-lang/antimony) project)
