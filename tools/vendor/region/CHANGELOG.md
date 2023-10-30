# Changelog

All notable changes to this project will be documented in this file.

## [Unreleased]

## [3.0.0] - 2021-08-05

### Added 

- Added support for OpenBSD.
- Added support for Illumos.
- Added support for memory allocations (`alloc` & `alloc_at`).
- Added `QueryIter` for lazily iterating regions.
- Added `inline` annotation where applicable.

### Changed

- Addresses are now defined as `*const T` (instead of `*const u8`).
- `Region` state has been internalized, now exposed via methods.
- `Error` enumerations have been altered.
- `query_iter` now returns an iterator.

### Removed

- Removed `page::size_from_range`.
- Removed deprecated functionality.

[unreleased]: https://github.com/darfink/region-rs/compare/v3.0.0...HEAD
[3.0.0]: https://github.com/darfink/region-rs/compare/v2.2.0...v3.0.0
