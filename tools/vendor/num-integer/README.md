# num-integer

[![crate](https://img.shields.io/crates/v/num-integer.svg)](https://crates.io/crates/num-integer)
[![documentation](https://docs.rs/num-integer/badge.svg)](https://docs.rs/num-integer)
[![minimum rustc 1.8](https://img.shields.io/badge/rustc-1.8+-red.svg)](https://rust-lang.github.io/rfcs/2495-min-rust-version.html)
[![build status](https://github.com/rust-num/num-integer/workflows/master/badge.svg)](https://github.com/rust-num/num-integer/actions)

`Integer` trait and functions for Rust.

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
num-integer = "0.1"
```

and this to your crate root:

```rust
extern crate num_integer;
```

## Features

This crate can be used without the standard library (`#![no_std]`) by disabling
the default `std` feature.  Use this in `Cargo.toml`:

```toml
[dependencies.num-integer]
version = "0.1.36"
default-features = false
```

There is no functional difference with and without `std` at this time, but
there may be in the future.

Implementations for `i128` and `u128` are only available with Rust 1.26 and
later.  The build script automatically detects this, but you can make it
mandatory by enabling the `i128` crate feature.

## Releases

Release notes are available in [RELEASES.md](RELEASES.md).

## Compatibility

The `num-integer` crate is tested for rustc 1.8 and greater.

## License

Licensed under either of

 * [Apache License, Version 2.0](http://www.apache.org/licenses/LICENSE-2.0)
 * [MIT license](http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
