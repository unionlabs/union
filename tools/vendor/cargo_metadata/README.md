# cargo_metadata

Structured access to the output of `cargo metadata`. Usually used from within a `cargo-*` executable.

Also supports serialization to aid in implementing `--message-format=json`-like
output generation in `cargo-*` subcommands, since some of the types in what
`cargo --message-format=json` emits are exactly the same as the ones from `cargo metadata`.

[![Build Status](https://api.travis-ci.org/oli-obk/cargo_metadata.svg?branch=master)](https://travis-ci.org/oli-obk/cargo_metadata)
[![crates.io](https://img.shields.io/crates/v/cargo_metadata.svg)](https://crates.io/crates/cargo_metadata)

[Documentation](https://docs.rs/cargo_metadata/)
