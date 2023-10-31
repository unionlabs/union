# `f16` and `bf16` floating point types for Rust
[![Crates.io](https://img.shields.io/crates/v/half.svg)](https://crates.io/crates/half/) [![Documentation](https://docs.rs/half/badge.svg)](https://docs.rs/half/) ![Crates.io](https://img.shields.io/crates/l/half) [![Build status](https://github.com/starkat99/half-rs/actions/workflows/rust.yml/badge.svg?branch=master)](https://github.com/starkat99/half-rs/actions/workflows/rust.yml)

This crate implements a half-precision floating point `f16` type for Rust implementing the IEEE
754-2008 standard [`binary16`](https://en.wikipedia.org/wiki/Half-precision_floating-point_format)
a.k.a `half` format, as well as a `bf16` type implementing the
[`bfloat16`](https://en.wikipedia.org/wiki/Bfloat16_floating-point_format) format.

## Usage

The `f16` and `bf16` types provides conversion operations as a normal Rust floating point type, but
since they are primarily leveraged for minimal floating point storage and most major hardware does
not implement them, all math operations are done as an `f32` type under the hood. Complex arithmetic
should manually convert to and from `f32` for better performance.

This crate provides [`no_std`](https://rust-embedded.github.io/book/intro/no-std.html) support by
default so can easily be used in embedded code where a smaller float format is most useful.

*Requires Rust 1.51 or greater.* If you need support for older versions of Rust, use versions 1.7.1
and earlier of this crate.

See the [crate documentation](https://docs.rs/half/) for more details.

### Optional Features

- **`serde`** - Implement `Serialize` and `Deserialize` traits for `f16` and `bf16`. This adds a
  dependency on the [`serde`](https://crates.io/crates/serde) crate.

- **`use-intrinsics`** - Use hardware intrinsics for `f16` and `bf16` conversions if available on
  the compiler host target. By default, without this feature, conversions are done only in software,
  which will be the fallback if the host target does not have hardware support. **Available only on
  Rust nightly channel.**

- **`alloc`** - Enable use of the [`alloc`](https://doc.rust-lang.org/alloc/) crate when not using
  the `std` library.

  This enables the `vec` module, which contains zero-copy conversions for the `Vec` type. This
  allows fast conversion between raw `Vec<u16>` bits and `Vec<f16>` or `Vec<bf16>` arrays, and vice
  versa.

- **`std`** - Enable features that depend on the Rust `std` library, including everything in the
  `alloc` feature.

  Enabling the `std` feature enables runtime CPU feature detection when the `use-intrsincis` feature
  is also enabled.
  Without this feature detection, intrinsics are only used when compiler host target supports them.

- **`num-traits`** - Enable `ToPrimitive`, `FromPrimitive`, `Num`, `Float`, `FloatCore` and
  `Bounded` trait implementations from the [`num-traits`](https://crates.io/crates/num-traits) crate.

- **`bytemuck`** - Enable `Zeroable` and `Pod` trait implementations from the
  [`bytemuck`](https://crates.io/crates/bytemuck) crate.

- **`zerocopy`** - Enable `AsBytes` and `FromBytes` trait implementations from the 
  [`zerocopy`](https://crates.io/crates/zerocopy) crate.

### More Documentation

- [Crate API Reference](https://docs.rs/half/)
- [Latest Changes](CHANGELOG.md)

## License

This library is distributed under the terms of either of:

* [MIT License](LICENSES/MIT.txt)
  ([http://opensource.org/licenses/MIT](http://opensource.org/licenses/MIT))
* [Apache License, Version 2.0](LICENSES/Apache-2.0.txt)
  ([http://www.apache.org/licenses/LICENSE-2.0](http://www.apache.org/licenses/LICENSE-2.0))

at your option.

This project is [REUSE-compliant](https://reuse.software/spec/). Copyrights are retained by their
contributors. Some files may include explicit copyright notices and/or license
[SPDX identifiers](https://spdx.dev/ids/). For full authorship information, see the version control
history.

### Contributing

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the
work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.
