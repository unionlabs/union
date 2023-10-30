# Adler-32 checksums for Rust

[![crates.io](https://img.shields.io/crates/v/adler.svg)](https://crates.io/crates/adler)
[![docs.rs](https://docs.rs/adler/badge.svg)](https://docs.rs/adler/)
![CI](https://github.com/jonas-schievink/adler/workflows/CI/badge.svg)

This crate provides a simple implementation of the Adler-32 checksum, used in
the zlib compression format.

Please refer to the [changelog](CHANGELOG.md) to see what changed in the last
releases.

## Features

- Permissively licensed (0BSD) clean-room implementation.
- Zero dependencies.
- Zero `unsafe`.
- Decent performance (3-4 GB/s).
- Supports `#![no_std]` (with `default-features = false`).

## Usage

Add an entry to your `Cargo.toml`:

```toml
[dependencies]
adler = "1.0.2"
```

Check the [API Documentation](https://docs.rs/adler/) for how to use the
crate's functionality.

## Rust version support

Currently, this crate supports all Rust versions starting at Rust 1.31.0.

Bumping the Minimum Supported Rust Version (MSRV) is *not* considered a breaking
change, but will not be done without good reasons. The latest 3 stable Rust
versions will always be supported no matter what.
