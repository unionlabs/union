# Fragile

[![Build Status](https://github.com/mitsuhiko/fragile/workflows/Tests/badge.svg?branch=master)](https://github.com/mitsuhiko/fragile/actions?query=workflow%3ATests)
[![Crates.io](https://img.shields.io/crates/d/fragile.svg)](https://crates.io/crates/fragile)
[![License](https://img.shields.io/github/license/mitsuhiko/fragile)](https://github.com/mitsuhiko/fragile/blob/master/LICENSE)
[![rustc 1.42.0](https://img.shields.io/badge/rust-1.42%2B-orange.svg)](https://img.shields.io/badge/rust-1.42%2B-orange.svg)
[![Documentation](https://docs.rs/fragile/badge.svg)](https://docs.rs/fragile)

This library provides wrapper types that permit sending non Send types to other
threads and use runtime checks to ensure safety.

It provides the `Fragile<T>`, `Sticky<T>` and `SemiSticky<T>` types which are
similar in nature but have different behaviors with regards to how destructors
are executed.  The `Fragile<T>` will panic if the destructor is called in another
thread, `Sticky<T>` will temporarily leak the object until the thread shuts down.
`SemiSticky<T>` is a compromise of the two.  It behaves like `Sticky<T>` but it
avoids the use of thread local storage if the type does not need `Drop`.

## Example

```rust
use std::thread;

// creating and using a fragile object in the same thread works
let val = Fragile::new(true);
assert_eq!(*val.get(), true);
assert!(val.try_get().is_ok());

// once send to another thread it stops working
thread::spawn(move || {
    assert!(val.try_get().is_err());
}).join()
    .unwrap();
```

## License and Links

- [Documentation](https://docs.rs/fragile/)
- [Issue Tracker](https://github.com/mitsuhiko/fragile/issues)
- License: [Apache 2.0](https://github.com/mitsuhiko/fragile/blob/master/LICENSE)
