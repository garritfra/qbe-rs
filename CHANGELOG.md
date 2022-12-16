# Changelog

All notable changes to this project will be documented in this file.

## Unreleased

### Changed

-   Various `new()` functions now take `Into<String>` instead of a
    `String` ([#15](https://github.com/garritfra/qbe-rs/pull/15))

## [2.1.0] - 2022-12-15

This release prepares the lib for the upcoming QBE 1.1.

### Added

-   `Type::size()` can now correctly calculate the size of aggregate types
    ([#12](https://github.com/garritfra/qbe-rs/pull/12)).
-   `Function::add_block()` returns a reference to the created block ([#18](https://github.com/garritfra/qbe-rs/pull/18))
- Add `blit` instruction, in preparation for QBE release 1.1 ([#20](https://github.com/garritfra/qbe-rs/pull/20)).

### Changed

-   `Type::Aggregate` now takes a `TypeDef` instead of the name of a type
    ([#12](https://github.com/garritfra/qbe-rs/pull/12)).
-   Deprecated `Function::last_block()` ([#18](https://github.com/garritfra/qbe-rs/pull/18))

## [2.0.0] - 2022-03-10

### Added

-   `Function` and `DataDef` now have a `new` constructor
-   `Module` now implements common traits (`Debug`, `Clone`, `Eq`, `PartialEq`,
    `Ord`, `PartialOrd`, `Hash`, `Default` and `Copy`)

### Changed

-   `Module::add_function`, `Module::add_type` and `Module::add_data` now consume
    their corresponding structs, instead of constructing them

## [1.0.0] - 2022-02-11

### Added

-   Data types now implement common traits (`Debug`, `Clone`, `Eq`, `PartialEq`,
    `Ord`, `PartialOrd`, `Hash`, `Default` and `Copy`) where applicable
-   Added `Linkage` data type (see [`Linkage`](https://c9x.me/compile/doc/il.html#Linkage))
-   Added a `Module` data type that houses functions and data definitions

### Changed

-   Remove `Qbe` prefix from data structures. `QbeValue` becomes `qbe::Value`
-   The `exported` flag of a `Function` has been replaced with `Linkage`

## [0.1.0] - 2022-02-09

### Added

-   Tests
-   Hello World example

### Changed

-   `QbeBlock` now has `statements` instead of `instructions`

## [0.0.1] - 2022-02-08

### Added

-   Initial release (taken over from the [Antimony](https://github.com/antimony-lang/antimony) project)
