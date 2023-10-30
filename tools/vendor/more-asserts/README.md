# [More Asserts](https://crates.io/crates/more-asserts) (for Rust).

[![Docs](https://docs.rs/more-asserts/badge.svg)](https://docs.rs/more-asserts)

Small library providing assertion macros similar to the `{debug_,}assert_{eq,ne}` macros in the stdlib.

## Usage

1. Add `more-asserts = "0.2"` to your `Cargo.toml`.
2. Add `#[macro_use] extern crate more_asserts` to your code.

After this, the following macros are available in your code (see [the documentation](https://docs.rs/more-asserts) for more info):

- `assert_lt!(left, right)`: Panics if `!(left < right)`. Optionally can take format arguments
- `assert_gt!(left, right)`: Panics if `!(left > right)`.
- `assert_le!(left, right)`: Panics if `!(left <= right)`.
- `assert_ge!(left, right)`: Panics if `!(left >= right)`.
- `debug_assert_lt!(left, right)`: Variant of `assert_lt!` controlled by `cfg!(debug_assertions)`.
- `debug_assert_gt!(left, right)`: Variant of `assert_gt!` controlled by `cfg!(debug_assertions)`.
- `debug_assert_le!(left, right)`: Variant of `assert_le!` controlled by `cfg!(debug_assertions)`.
- `debug_assert_ge!(left, right)`: Variant of `assert_ge!` controlled by `cfg!(debug_assertions)`.
- `debug_unreachable!(...)`: Variant of the standard library's `unreachable!`
  that is controlled by `cfg!(debug_assertations)`.

Note that `assert_eq!`, `assert_ne!`, `debug_assert_eq!`, and `debug_assert_ne!` are not provided, as those are in the standard library.

## License

[CC0 (public domain)](https://creativecommons.org/publicdomain/zero/1.0/).
