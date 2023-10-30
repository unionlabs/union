memchr
======
This library provides heavily optimized routines for string search primitives.

[![Build status](https://github.com/BurntSushi/memchr/workflows/ci/badge.svg)](https://github.com/BurntSushi/memchr/actions)
[![Crates.io](https://img.shields.io/crates/v/memchr.svg)](https://crates.io/crates/memchr)

Dual-licensed under MIT or the [UNLICENSE](https://unlicense.org/).


### Documentation

[https://docs.rs/memchr](https://docs.rs/memchr)


### Overview

* The top-level module provides routines for searching for 1, 2 or 3 bytes
  in the forward or reverse direction. When searching for more than one byte,
  positions are considered a match if the byte at that position matches any
  of the bytes.
* The `memmem` sub-module provides forward and reverse substring search
  routines.

In all such cases, routines operate on `&[u8]` without regard to encoding. This
is exactly what you want when searching either UTF-8 or arbitrary bytes.

### Compiling without the standard library

memchr links to the standard library by default, but you can disable the
`std` feature if you want to use it in a `#![no_std]` crate:

```toml
[dependencies]
memchr = { version = "2", default-features = false }
```

On x86 platforms, when the `std` feature is disabled, the SSE2 accelerated
implementations will be used. When `std` is enabled, AVX accelerated
implementations will be used if the CPU is determined to support it at runtime.

### Using libc

`memchr` is a routine that is part of libc, although this crate does not use
libc by default. Instead, it uses its own routines, which are either vectorized
or generic fallback routines. In general, these should be competitive with
what's in libc, although this has not been tested for all architectures. If
using `memchr` from libc is desirable and a vectorized routine is not otherwise
available in this crate, then enabling the `libc` feature will use libc's
version of `memchr`.

The rest of the functions in this crate, e.g., `memchr2` or `memrchr3` and the
substring search routines, will always use the implementations in this crate.
One exception to this is `memrchr`, which is an extension in `libc` found on
Linux. On Linux, `memrchr` is used in precisely the same scenario as `memchr`,
as described above.


### Minimum Rust version policy

This crate's minimum supported `rustc` version is `1.41.1`.

The current policy is that the minimum Rust version required to use this crate
can be increased in minor version updates. For example, if `crate 1.0` requires
Rust 1.20.0, then `crate 1.0.z` for all values of `z` will also require Rust
1.20.0 or newer. However, `crate 1.y` for `y > 0` may require a newer minimum
version of Rust.

In general, this crate will be conservative with respect to the minimum
supported version of Rust.


### Testing strategy

Given the complexity of the code in this crate, along with the pervasive use
of `unsafe`, this crate has an extensive testing strategy. It combines multiple
approaches:

* Hand-written tests.
* Exhaustive-style testing meant to exercise all possible branching and offset
  calculations.
* Property based testing through [`quickcheck`](https://github.com/BurntSushi/quickcheck).
* Fuzz testing through [`cargo fuzz`](https://github.com/rust-fuzz/cargo-fuzz).
* A huge suite of benchmarks that are also run as tests. Benchmarks always
  confirm that the expected result occurs.

Improvements to the testing infrastructure are very welcome.


### Algorithms used

At time of writing, this crate's implementation of substring search actually
has a few different algorithms to choose from depending on the situation.

* For very small haystacks,
  [Rabin-Karp](https://en.wikipedia.org/wiki/Rabin%E2%80%93Karp_algorithm)
  is used to reduce latency. Rabin-Karp has very small overhead and can often
  complete before other searchers have even been constructed.
* For small needles, a variant of the
  ["Generic SIMD"](http://0x80.pl/articles/simd-strfind.html#algorithm-1-generic-simd)
  algorithm is used. Instead of using the first and last bytes, a heuristic is
  used to select bytes based on a background distribution of byte frequencies.
* In all other cases,
  [Two-Way](https://en.wikipedia.org/wiki/Two-way_string-matching_algorithm)
  is used. If possible, a prefilter based on the "Generic SIMD" algorithm
  linked above is used to find candidates quickly. A dynamic heuristic is used
  to detect if the prefilter is ineffective, and if so, disables it.
