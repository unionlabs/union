# encode_unicode

UTF-8 and UTF-16 character types, iterators and related methods for `char`, `u8` and `u16`.

[![crates.io page](https://img.shields.io/crates/v/encode_unicode.svg)](https://crates.io/crates/encode_unicode/)

## Features

* **[`Utf8Char`](https://docs.rs/encode_unicode/latest/encode_unicode/struct.Utf8Char.html)**:
  A `char` stored as UTF-8. Can be borrowed as a `str` or `u8` slice.
* **[`Utf16Char`](https://docs.rs/encode_unicode/latest/encode_unicode/struct.Utf16Char.html)**:
  A `char` stored as UTF-16. Can be borrowed as an `u16` slice.
* [Conversion methods on `char`](https://docs.rs/encode_unicode/latest/encode_unicode/trait.CharExt.html):
  * to and from UTF-8 as `[u8; 4]` or slice.
  * to and from UTF-16 as `(u16, Option<u16>)` or slice.
* [Iterator adapters](https://docs.rs/encode_unicode/latest/encode_unicode/trait.IterExt.html)
  for converting betwenn `u8`s and `Utf8Char`s or `u16`s and `Utf16Char`s.
* Optimized [slice-based decoding iterators](https://docs.rs/encode_unicode/latest/encode_unicode/trait.SliceExt.html).
* [Precise errors when decoding a char from UTF-8, UTF-16 or `u32` fails](http://docs.rs/encode_unicode/latest/encode_unicode/error/index.html).
* Utility methods on [`u8`](https://docs.rs/encode_unicode/latest/encode_unicode/trait.U8UtfExt.html)
  and [`u16`](https://docs.rs/encode_unicode/latest/encode_unicode/trait.U16UtfExt.html).

The minimum supported version of Rust is 1.15,
older versions might work now but can break with a minor update.

## Optional features

* `#![no_std]`-mode: There are a few differences:
  * `Error` doesn't exist, but `description()` is made available as an inherent impl.
  * `Extend`/`FromIterator`-implementations for `String`/`Vec<u8>`/`Vec<u16>` are missing.
  * There is no `io`, so `Utf8Iterator` and `Utf8CharSplitter` doesn't implement `Read`.
  This feature is enabled by setting `default-features=false` in `Cargo.toml`:
  `encode_unicode = {version="0.3.4", default-features=false}`.
* Integration with the [ascii](https://tomprogrammer.github.io/rust-ascii/ascii/index.html) crate:
  Convert `Utf8Char` and `Utf16Char` to and from [ascii::`AsciiChar`](https://tomprogrammer.github.io/rust-ascii/ascii/enum.AsciiChar.html).

## License

Licensed under either of

* Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the work by you, as defined in the Apache-2.0
license, shall be dual licensed as above, without any additional terms or
conditions.

## History

The original purpose of this crate was to provide standins for the then
unstable `encode_utf8()` and `encode_utf16()`.  
The standins were removed in 0.3 when Rust 1.15 stabilized the `encode_`
methods, but the other stuff I added, such as iterators like
those `encode_utf{8,16}() returned for a while, might still be of use.
