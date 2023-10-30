# rand_xoshiro

[![Build Status](https://travis-ci.org/rust-random/rngs.svg?branch=master)](https://travis-ci.org/rust-random/rngs)
[![Build Status](https://ci.appveyor.com/api/projects/status/github/rust-random/rngs?svg=true)](https://ci.appveyor.com/project/rust-random/rngs)
[![Latest version](https://img.shields.io/crates/v/rand_xoshiro.svg)](https://crates.io/crates/rand_xoshiro)
[![Book](https://img.shields.io/badge/book-master-yellow.svg)](https://rust-random.github.io/book/)
[![API](https://img.shields.io/badge/api-master-yellow.svg)](https://rust-random.github.io/rand/rand_xoshiro)
[![API](https://docs.rs/rand_xoshiro/badge.svg)](https://docs.rs/rand_xoshiro)
[![Minimum rustc version](https://img.shields.io/badge/rustc-1.32+-lightgray.svg)](https://github.com/rust-random/rngs#rust-version-requirements)

Rust implementation of the [xoshiro, xoroshiro and splitmix64](http://xoshiro.di.unimi.it) random number generators.

This crate depends on [rand_core](https://crates.io/crates/rand_core) and is
part of the [Rand project](https://github.com/rust-random/rand).

Links:

-   [API documentation (master)](https://rust-random.github.io/rand/rand_xoshiro)
-   [API documentation (docs.rs)](https://docs.rs/rand_xoshiro)
-   [Changelog](https://github.com/rust-random/rngs/blob/master/rand_xoshiro/CHANGELOG.md)

## Crate Features

`rand_xoshiro` is no_std compatible by default.

The `serde1` feature includes implementations of `Serialize` and `Deserialize` for the included RNGs.

## License

`rand_xoshiro` is distributed under the terms of both the MIT license and the
Apache License (Version 2.0).

See [LICENSE-APACHE](LICENSE-APACHE) and [LICENSE-MIT](LICENSE-MIT), and
[COPYRIGHT](COPYRIGHT) for details.
