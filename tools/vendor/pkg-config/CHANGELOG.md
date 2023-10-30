# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/)
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.3.25] - 2022-03-31

### Added

- Support for parsing `-Wl` linker arguments from the `Libs` lines and
  passing them to the linker as well as making them available via
  `Library::ld_args` (#131).

### Changed

- Use SPDX license format and remove obsolete badge info (#129).

## [0.3.24] - 2021-12-11

### Fixed

- Re-add `target_supported()`, which was accidentally removed in 0.3.15 (#128).

## [0.3.23] - 2021-12-06

### Changed

- Improve error messages when a `pkg-config` package can't be found (#127).

## [0.3.22] - 2021-10-24

### Fixed

- `pkg-config` compiles again with Rust 1.30 or newer. 0.3.21 accidentally
  made use of API only available since 1.40 (#124, #125).

### Changed

- Switched from Travis to GitHub Actions for the CI. Travis is dysfunctional
  since quite some time (#126).

## [0.3.21] - 2021-10-22

### Fixed

- Tests succeed again on macOS (#122).

### Changed

- Improve error message in case of missing pkg-config and provide instructions
  how it can be installed (#121).

## [0.3.20] - 2021-09-25

### Fixed

- Use target-specific pkg-config consistently everywhere (#121, #118).

## [0.3.19] - 2020-10-13

### Added

- Add `README.md` to be displayed on crates.io (#111).

- Support for `-isystem`, `-iquote` and `-idirafter` include flags (#115).

### Changed

- Improve documentation for cross-compilation (#113).

- Allow overriding system root via the `PKG_CONFIG_SYSROOT_DIR` or `SYSROOT`
  environment variable (#82).

## [0.3.18] - 2020-07-11

### Fixed

- Use `env::var_os()` almost everywhere to handle non-UTF8 paths in
  environment variables, and also improve error handling around environment
  variable handling (#106).

### Changed

- Default the `env_metadata` build parameter to `true` instead of `false`.
  Whenever a pkg-config related environment variable changes it would make
  sense to rebuild crates that use pkg-config, or otherwise changes might not
  be picked up. As such the previous default didn't make much sense (#105).

## [0.3.17] - 2019-11-02

### Fixed

- Fix support for multiple version number constraints (#95)

## [0.3.16] - 2019-09-09

### Changed
- Stop using deprecated functions and require Rust 1.30 (#84)

### Fixed
- Fix repository URL in README.md
- Fix various clippy warnings

### Added
- Run `cargo fmt` as part of the CI (#89)
- Derive `Clone` for `Library` and `Debug` for `Config (#91)
- Add support for `PKG_CONFIG_ALLOW_SYSTEM_CFLAGS` and enable by default (#93)

## [0.3.15] - 2019-07-25

### Changed
- Changes minimum documented rust version to 1.28 (#76)

### Fixed
- Fix Travis CI badge url (#78)
- Fix project name in README.md (#81)

### Added
- Support specifying range of versions (#75)
- Allow cross-compilation if pkg-config is customized (#44, #86)

## [0.3.14] - 2018-08-28

### Fixed
- Don't append .lib suffix on MSVC builds (#72)

## [0.3.13] - 2018-08-06

### Fixed
- Fix MSVC support to actually work and consider library paths too (#71)

## [0.3.12] - 2018-06-18

### Added
- Support for MSVC (#70)
- Document and test Rust 1.13 as minimally supported version (#66)

## [0.3.11] - 2018-04-24

### Fixed
- Re-added AsciiExt import (#65)

## [0.3.10] - 2018-04-23

### Added
- Allow static linking of /usr/ on macOS (#42)
- Add support for parsing `-Wl,` style framework flags (#48)
- Parse defines in `pkg-config` output (#49)
- Rerun on `PKG_CONFIG_PATH` changes (#50)
- Introduce target-scoped variables (#58)
- Respect pkg-config escaping rules used with --cflags and --libs (#61)

### Changed
- Use `?` instead of `try!()` in the codebase (#63)
