# Changelog

## Unreleased

No changes.

## [1.0.2 - 2021-02-26](https://github.com/jonas-schievink/adler/releases/tag/v1.0.2)

- Fix doctest on big-endian systems ([#9]).

[#9]: https://github.com/jonas-schievink/adler/pull/9

## [1.0.1 - 2020-11-08](https://github.com/jonas-schievink/adler/releases/tag/v1.0.1)

### Fixes

- Fix documentation on docs.rs.

## [1.0.0 - 2020-11-08](https://github.com/jonas-schievink/adler/releases/tag/v1.0.0)

### Fixes

- Fix `cargo test --no-default-features` ([#5]).

### Improvements

- Extended and clarified documentation.
- Added more rustdoc examples.
- Extended CI to test the crate with `--no-default-features`.

### Breaking Changes

- `adler32_reader` now takes its generic argument by value instead of as a `&mut`.
- Renamed `adler32_reader` to `adler32`.

## [0.2.3 - 2020-07-11](https://github.com/jonas-schievink/adler/releases/tag/v0.2.3)

- Process 4 Bytes at a time, improving performance by up to 50% ([#2]).

## [0.2.2 - 2020-06-27](https://github.com/jonas-schievink/adler/releases/tag/v0.2.2)

- Bump MSRV to 1.31.0.

## [0.2.1 - 2020-06-27](https://github.com/jonas-schievink/adler/releases/tag/v0.2.1)

- Add a few `#[inline]` annotations to small functions.
- Fix CI badge.
- Allow integration into libstd.

## [0.2.0 - 2020-06-27](https://github.com/jonas-schievink/adler/releases/tag/v0.2.0)

- Support `#![no_std]` when using `default-features = false`.
- Improve performance by around 7x.
- Support Rust 1.8.0.
- Improve API naming.

## [0.1.0 - 2020-06-26](https://github.com/jonas-schievink/adler/releases/tag/v0.1.0)

Initial release.


[#2]: https://github.com/jonas-schievink/adler/pull/2
[#5]: https://github.com/jonas-schievink/adler/pull/5
