[![License:Zlib](https://img.shields.io/badge/License-Zlib-brightgreen.svg)](https://opensource.org/licenses/Zlib)
![Minimum Rust Version](https://img.shields.io/badge/Min%20Rust-1.34-green.svg)
[![crates.io](https://img.shields.io/crates/v/tinyvec.svg)](https://crates.io/crates/tinyvec)
[![docs.rs](https://docs.rs/tinyvec/badge.svg)](https://docs.rs/tinyvec/)

![Unsafe-Zero-Percent](https://img.shields.io/badge/Unsafety-0%25-brightgreen.svg)

# tinyvec

A 100% safe crate of vec-like types. `#![forbid(unsafe_code)]`

Main types are as follows:
* `ArrayVec` is an array-backed vec-like data structure. It panics on overflow.
* `SliceVec` is the same deal, but using a `&mut [T]`.
* `TinyVec` (`alloc` feature) is an enum that's either an `Inline(ArrayVec)` or a `Heap(Vec)`. If a `TinyVec` is `Inline` and would overflow it automatically transitions to `Heap` and continues whatever it was doing.

To attain this "100% safe code" status there is one compromise: the element type of the vecs must implement `Default`.

For more details, please see [the docs.rs documentation](https://docs.rs/tinyvec/)
