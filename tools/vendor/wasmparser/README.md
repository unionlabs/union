# The WebAssembly binary file decoder in Rust

**A [Bytecode Alliance](https://bytecodealliance.org/) project**

[![crates.io link](https://img.shields.io/crates/v/wasmparser.svg)](https://crates.io/crates/wasmparser)
[![docs.rs docs](https://img.shields.io/static/v1?label=docs&message=wasmparser&color=blue&style=flat-square)](https://docs.rs/wasmparser/)

The decoder library provides lightweight and fast decoding/parsing of WebAssembly binary files.

The other goal is minimal memory footprint. For this reason, there is no AST or IR of WebAssembly data.

See also its sibling at https://github.com/wasdk/wasmparser


## Documentation

The documentation and examples can be found at the https://docs.rs/wasmparser/


## Fuzzing

To fuzz test wasmparser.rs, switch to a nightly Rust compiler and install [cargo-fuzz]:

```
cargo install cargo-fuzz
```

Then, from the root of the repository, run:

```
cargo fuzz run parse
```

If you want to use files as seeds for the fuzzer, add them to `fuzz/corpus/parse/` and restart cargo-fuzz.

[cargo-fuzz]: https://github.com/rust-fuzz/cargo-fuzz
