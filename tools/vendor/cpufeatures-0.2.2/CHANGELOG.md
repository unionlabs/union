# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## 0.2.2 (2022-03-18)
### Added
- Support for Android on `aarch64` ([#752])

### Removed
- Vestigial code around `crypto` target feature ([#600])

[#600]: https://github.com/RustCrypto/utils/pull/600
[#752]: https://github.com/RustCrypto/utils/pull/752

## 0.2.1 (2021-08-26)
### Changed
- Revert [#583] "Use from_bytes_with_nul for string check" ([#597])

[#583]: https://github.com/RustCrypto/utils/pull/583
[#597]: https://github.com/RustCrypto/utils/pull/597

## 0.2.0 (2021-08-26) [YANKED]
### Removed
- AArch64 `crypto` target feature ([#594])

[#594]: https://github.com/RustCrypto/utils/pull/594

## 0.1.5 (2021-06-21)
### Added
- iOS support ([#435], [#501])

### Changed
- Map `aarch64` HWCAPs to target features; add `crypto` ([#456])

[#435]: https://github.com/RustCrypto/utils/pull/435
[#456]: https://github.com/RustCrypto/utils/pull/456
[#501]: https://github.com/RustCrypto/utils/pull/501

## 0.1.4 (2021-05-14)
### Added
- Support compiling on non-Linux/macOS aarch64 targets ([#408])

[#408]: https://github.com/RustCrypto/utils/pull/408

## 0.1.3 (2021-05-13)
### Removed
- `neon` on `aarch64` targets: already enabled by default ([#406])

[#406]: https://github.com/RustCrypto/utils/pull/406

## 0.1.2 (2021-05-13) [YANKED]
### Added
- `neon` feature detection on `aarch64` targets ([#403])

### Fixed
- Support for `musl`-based targets ([#403])

[#403]: https://github.com/RustCrypto/utils/pull/403

## 0.1.1 (2021-05-06)
### Added
- `aarch64` support for Linux and macOS/M4 targets ([#393])

[#393]: https://github.com/RustCrypto/utils/pull/393

## 0.1.0 (2021-04-29)
- Initial release
