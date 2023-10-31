# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## 0.10.6 (2022-12-16)
### Added
- `size_opt` Cargo feature ([#440])

### Changed
- Implement `KeyInit::new` in terms of `KeyInit::new_from_slice` ([#435])

[#435]: https://github.com/RustCrypto/hashes/pull/435
[#440]: https://github.com/RustCrypto/hashes/pull/440

## 0.10.5 (2022-11-11)
### Fixed
- Implementation of the `KeyInit::new` method for MAC types ([#432])

[#432]: https://github.com/RustCrypto/hashes/pull/432

## 0.10.4 (2022-02-17) [YANKED]
### Fixed
- Bug on big-endian targets ([#366])

[#366]: https://github.com/RustCrypto/hashes/pull/366

## 0.10.3 (2022-02-17) [YANKED]
### Fixed
- Minimal versions build ([#363])

[#363]: https://github.com/RustCrypto/hashes/pull/363

## 0.10.2 (2022-01-09) [YANKED]
## Fixed
- Rare compilation error by adding `'static` bound on `OutSize`. ([#347])
- Values of `KeySize` associated type. ([#349])

[#347]: https://github.com/RustCrypto/hashes/pull/347
[#349]: https://github.com/RustCrypto/hashes/pull/349

## 0.10.1 (2022-01-05) [YANKED]
## Fixed
- Compilation error with enabled `reset` feature. ([#342])

[#342]: https://github.com/RustCrypto/hashes/pull/342

## 0.10.0 (2021-12-07) [YANKED]
### Changed
- Update to `digest` v0.10 and remove dependency on `crypto-mac` ([#217])
- `Blake2b` and `Blake2s` renamed into `Blake2b512` and `Blake2s256` respectively.
  New `Blake2b` and `Blake2s` are generic over output size. `VarBlake2b` and `VarBlake2s`
  renamed into `Blake2bVar` and `Blake2sVar` respectively. ([#217])
- Hasher reset functionality moved behind a new non-default feature, `reset`.
  This must be enabled to use the methods `reset`, `finalize_reset` and `finalize_into_reset`.

### Removed
- `Blake2b` and `Blake2s` no longer support MAC functionality. ([#217])

### Added
- Separate `Blake2bMac` and `Blake2sMac` types generic over output size and `Blake2bMac512`
  and `Blake2sMac256` type aliases around them. ([#217])

[#217]: https://github.com/RustCrypto/hashes/pull/217

## 0.9.2 (2021-08-25)
### Fixed
- Building with `simd_opt` on recent nightlies ([#301]) 

[#301]: https://github.com/RustCrypto/hashes/pull/301

## 0.9.1 (2020-10-26)
### Changed
- Bump `opaque-debug` to v0.3 ([#168])
- Bump `block-buffer` to v0.9 ([#164])

[#168]: https://github.com/RustCrypto/hashes/pull/168
[#164]: https://github.com/RustCrypto/hashes/pull/164

## 0.9.0 (2020-06-10)
### Added
- Support for Persona and Salt ([#78]) 

### Changed
- Update to `digest` v0.9 release; MSRV 1.41+ ([#155])
- Use new `*Dirty` traits from the `digest` crate ([#153])
- Bump `crypto-mac` to v0.8 release ([#152])
- Bump `block-buffer` to v0.8 release ([#151])
- Rename `*result*` to `finalize` ([#148])
- Upgrade to Rust 2018 edition ([#119])

[#155]: https://github.com/RustCrypto/hashes/pull/155
[#153]: https://github.com/RustCrypto/hashes/pull/153
[#152]: https://github.com/RustCrypto/hashes/pull/152
[#151]: https://github.com/RustCrypto/hashes/pull/151
[#148]: https://github.com/RustCrypto/hashes/pull/148
[#119]: https://github.com/RustCrypto/hashes/pull/133
[#78]: https://github.com/RustCrypto/hashes/pull/78

## 0.8.1 (2019-08-25)

## 0.8.0 (2018-10-11)

## 0.7.1 (2018-04-30)

## 0.7.0 (2017-11-15)

## 0.6.1 (2017-07-24)

## 0.6.0 (2017-06-12)

## 0.5.2 (2017-05-17)

## 0.5.1 (2017-05-02)

## 0.5.0 (2017-04-06)

## 0.4.0 (2017-03-06)

## 0.3.0 (2016-11-17)

## 0.2.0 (2016-10-14)

## 0.1.1 (2016-10-11)

## 0.1.0 (2016-10-09)
