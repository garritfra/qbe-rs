# Changelog

All notable changes to this project will be documented in this file.

## Unreleased

### Added

- Pin Rust toolchain to 1.88 in `rust-toolchain.toml` ([#37](https://github.com/garritfra/qbe-rs/pull/37))
- BREAKING: Support for opaque and union types ([#39](https://github.com/garritfra/qbe-rs/pull/39))

## [2.5.0] - 2025-04-14

### Added

- Code documentation examples
- Alignment calculation for type definition ([#33](https://github.com/garritfra/qbe-rs/pull/33))
- Phi instruction ([#34](https://github.com/garritfra/qbe-rs/pull/34))

### Fixed

- Correct size calculation for type definition ([#33](https://github.com/garritfra/qbe-rs/pull/33))

## [2.4.0] - 2025-02-28

### Added

- Additional comparison operators: ordered (O), unordered (UO), and unsigned integer comparisons (Ult, Ule, Ugt, Uge)
- Bitwise shifting instructions: `Sar`, `Shr`, and `Shl`
- Unsigned arithmetic instructions: `Udiv` and `Urem`
- Type conversion instructions:
  - `Cast` instruction for converting between integers and floating points
  - Extension operations: `Extsw`, `Extuw`, `Extsh`, `Extuh`, `Extsb`, `Extub`
  - Float precision conversion: `Exts`, `Truncd`
  - Float-integer conversions: `Stosi`, `Stoui`, `Dtosi`, `Dtoui`, `Swtof`, `Uwtof`, `Sltof`, `Ultof`
- Variadic function support with `Vastart` and `Vaarg` instructions
- Program termination instruction `Hlt`
- Thread-local storage support in `Linkage` with convenience constructors
- Zero-initialized data support with `DataItem::Zero`

## [2.3.1] - 2025-02-28

### Fixed

- Fixed type definition ordering in `Module::fmt::Display` to ensure type definitions appear before function definitions, which is required by QBE for aggregate types ([#31](https://github.com/garritfra/qbe-rs/pull/31)).

## [2.3.0] - 2025-01-13

### Added

-   New `Block::add_comment` API to add comments inside blocks; `Block::items` is now `Vec<BlockItem>` instead of `Vec<Statement>` ([#25](https://github.com/garritfra/qbe-rs/pull/25)).
-   New `Type::Zero` for internal zero-sized type representation. ([#27](https://github.com/garritfra/qbe-rs/pull/27))
-   Debug instruction support with `Instr::DbgFile` and `Instr::DbgLoc` for source mapping. ([#27](https://github.com/garritfra/qbe-rs/pull/27))

### Changed

-   BREAKING: New field `Option<u64>` inside `Instr::Call` to specify variadic arguments ([#24](https://github.com/garritfra/qbe-rs/pull/24)).

## [2.2.0] - 2024-10-28

### Changed

-   Various `new()` functions now take `Into<String>` instead of a
    `String` ([#15](https://github.com/garritfra/qbe-rs/pull/15))
-   Add unsigned and signed variants of sub-word types: `Type::SignedByte`, `Type::UnsignedByte`, `Type::SignedHalfword`, `Type::UnsignedHalfword` ([#23](https://github.com/garritfra/qbe-rs/pull/23))

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
