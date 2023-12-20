# `wasmer-compiler-cranelift` [![Build Status](https://github.com/wasmerio/wasmer/workflows/build/badge.svg?style=flat-square)](https://github.com/wasmerio/wasmer/actions?query=workflow%3Abuild) [![Join Wasmer Slack](https://img.shields.io/static/v1?label=Slack&message=join%20chat&color=brighgreen&style=flat-square)](https://slack.wasmer.io) [![MIT License](https://img.shields.io/github/license/wasmerio/wasmer.svg?style=flat-square)](https://github.com/wasmerio/wasmer/blob/master/LICENSE) [![crates.io](https://img.shields.io/crates/v/wasmer-compiler-cranelift.svg)](https://crates.io/crates/wasmer-compiler-cranelift)

This crate contains a compiler implementation based on Cranelift.

## Usage

```rust
use wasmer::{Store, EngineBuilder};
use wasmer_compiler_cranelift::Cranelift;

let compiler = Cranelift::new();
let mut store = Store::new(compiler);
```

*Note: you can find a [full working example using Cranelift compiler
here][example].*

## When to use Cranelift

We recommend using this compiler crate **only for development
proposes**. For production we recommend using [`wasmer-compiler-llvm`]
as it offers a much better runtime speed (50% faster on average).

### Acknowledgments

This project borrowed some of the function lowering from
[`cranelift-wasm`].

Please check [Wasmer `ATTRIBUTIONS`] to further see licenses and other
attributions of the project.


[example]: https://github.com/wasmerio/wasmer/blob/master/examples/compiler_cranelift.rs
[`wasmer-compiler-llvm`]: https://github.com/wasmerio/wasmer/tree/master/lib/compiler-llvm
[`cranelift-wasm`]: https://crates.io/crates/cranelift-wasm
[Wasmer `ATTRIBUTIONS`]: https://github.com/wasmerio/wasmer/blob/master/ATTRIBUTIONS.md
