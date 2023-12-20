# Change Log

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

Changelog entries will contain a link to the pull request implementing that
change, where applicable.

<!-- next-header -->

## [Unreleased] - ReleaseDate

## [0.1.3] - 2023-06-07

- Added an `impl From<OwnedBuffer> for Vec<u8>`
  ([#3](https://github.com/wasmerio/shared-buffer/pull/3))

## [0.1.2] - 2023-06-06

### Fixed

- When slicing a mmapped buffer, the resulting offsets would be relative to the
  start of the file and not the start of the slice
  ([#1](https://github.com/wasmerio/shared-buffer/pull/1))

## [0.1.1] - 2023-05-18

### Added

- Added more `PartialEq` trait implementations to `OwnedBuffer`

## [0.1.0] - 2023-05-18

### Added

- Created an `OwnedBuffer` type which is either an in-memory buffer
  (`bytes::Bytes`) or a mmapped buffer (`memmap2::Mmap`)

<!-- next-url -->
[Unreleased]: https://github.com/wasmerio/shared-buffer/compare/v0.1.3...HEAD
[0.1.3]: https://github.com/wasmerio/shared-buffer/compare/v0.1.2...v0.1.3
[0.1.2]: https://github.com/wasmerio/shared-buffer/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/wasmerio/shared-buffer/compare/v0.1.0..v0.1.1
[0.1.0]: https://github.com/wasmerio/shared-buffer/compare/6c299238..v0.1.0
