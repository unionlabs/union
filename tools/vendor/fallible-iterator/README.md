# rust-fallible-iterator

[![CircleCI](https://circleci.com/gh/sfackler/rust-fallible-iterator.svg?style=shield)](https://circleci.com/gh/sfackler/rust-fallible-iterator)

[Documentation](https://sfackler.github.io/rust-fallible-iterator/doc/v0.1.3/fallible_iterator)

"Fallible" iterators for Rust.

## Features

If the `std` or `alloc` features are enabled, this crate provides implementations for
`Box`, `Vec`, `BTreeMap`, and `BTreeSet`. If the `std` feature is enabled, this crate
additionally provides implementations for `HashMap` and `HashSet`.

If the `std` feature is disabled, this crate does not depend on `libstd`.
