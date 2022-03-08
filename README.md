# QBE IR for Rust

[![](https://img.shields.io/crates/v/qbe.svg)](https://crates.io/crates/qbe)
[![docs](https://img.shields.io/badge/docs-docs.rs-blue.svg)](https://docs.rs/qbe)
[![Crates.io](https://img.shields.io/crates/l/qbe)](https://github.com/garritfra/qbe-rs/blob/main/COPYRIGHT)

https://c9x.me/compile/

This crate seeks to provide a Rust-y representation of [QBE
IR](https://c9x.me/compile/). It can be used for code generation of compilers. A
way to parse existing IR is planned.

## Getting Started

This crate is on [crates.io](https://crates.io/crates/qbe), so you can simply
add it as a dependency in your Cargo.toml and off you go.

If you don't know where to get started, check out the `hello_world` example in
the `examples/` directory.

## Projects using this crate

This crate is used by the [Antimony](https://github.com/antimony-lang/antimony)
project. Check out the [QBE
generator](https://github.com/antimony-lang/antimony/blob/master/src/generator/qbe.rs)
to see how they are using it.

## License

The `qbe` crate is distributed under either of

-   [Apache License, Version 2.0](LICENSE-APACHE)
-   [MIT license](LICENSE-MIT)

at your option.
