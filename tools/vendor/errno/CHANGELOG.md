# [Unreleased]

# [0.3.1] - 2023-04-08

- Correct link name on redox
  [#69](https://github.com/lambda-fairy/rust-errno/pull/69)

- Update windows-sys requirement from 0.45 to 0.48
  [#70](https://github.com/lambda-fairy/rust-errno/pull/70)

# [0.3.0] - 2023-02-12

- Add haiku support
  [#42](https://github.com/lambda-fairy/rust-errno/pull/42)

- Add AIX support
  [#54](https://github.com/lambda-fairy/rust-errno/pull/54)

- Add formatting with `#![no_std]`
  [#44](https://github.com/lambda-fairy/rust-errno/pull/44)

- Switch from `winapi` to `windows-sys` [#55](https://github.com/lambda-fairy/rust-errno/pull/55)

- Update minimum Rust version to 1.48
  [#48](https://github.com/lambda-fairy/rust-errno/pull/48) [#55](https://github.com/lambda-fairy/rust-errno/pull/55)

- Upgrade to Rust 2018 edition [#59](https://github.com/lambda-fairy/rust-errno/pull/59)

- wasm32-wasi: Use `__errno_location` instead of `feature(thread_local)`. [#66](https://github.com/lambda-fairy/rust-errno/pull/66)

# [0.2.8] - 2021-10-27

- Optionally support no_std
  [#31](https://github.com/lambda-fairy/rust-errno/pull/31)

[Unreleased]: https://github.com/lambda-fairy/rust-errno/compare/v0.3.1...HEAD
[0.3.1]: https://github.com/lambda-fairy/rust-errno/compare/v0.3.0...v0.3.1
[0.3.0]: https://github.com/lambda-fairy/rust-errno/compare/v0.2.8...v0.3.0
[0.2.8]: https://github.com/lambda-fairy/rust-errno/compare/v0.2.7...v0.2.8
