# Changelog

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.0.9] - 2022-05-19

### Fixed

- Documentation fixes.

## [1.0.8] - 2022-05-09

### Added

- New methods `canonicalize_utf8`, `read_link_utf8` and `read_dir_utf8` return `Utf8PathBuf`s, erroring out if a resulting path is not valid UTF-8.
- New feature `proptest1` introduces proptest `Arbitrary` impls for `Utf8PathBuf` and
  `Box<Utf8Path>` ([#18], thanks [mcronce](https://github.com/mcronce) for your first contribution!)
  
[#18]: https://github.com/camino-rs/camino/pull/18

## [1.0.7] - 2022-01-16

### Added

- `Utf8Path::is_symlink` checks whether a path is a symlink. Note that while `std::path::Path` only
  provides this method for version 1.58 and above, `camino` polyfills the method for all Rust versions
  it supports.

### Changed

- Update repository links to new location [camino-rs/camino](https://github.com/camino-rs/camino).
- Update `structopt` example to clap 3's builtin derive feature.
  (camino continues to work with structopt as before.)

## [1.0.6] - 2022-01-16

(This release was yanked due to a publishing issue.)

## [1.0.5] - 2021-07-27

### Added

- `Utf8PathBuf::into_std_path_buf` converts a `Utf8PathBuf` to a `PathBuf`; equivalent to the
  `From<Utf8PathBuf> for PathBuf` impl, but may aid in type inference.
- `Utf8Path::as_std_path` converts a `Utf8Path` to a `Path`; equivalent to the
  `AsRef<&Path> for &Utf8Path` impl, but may aid in type inference.

## [1.0.4] - 2021-03-19

### Fixed

- `Hash` impls for `Utf8PathBuf` and `Utf8Path` now match as required by the `Borrow` contract ([#9]).

[#9]: https://github.com/camino-rs/camino/issues/9

## [1.0.3] - 2021-03-11

### Added

- `TryFrom<PathBuf> for Utf8PathBuf` and `TryFrom<&Path> for &Utf8Path`, both of which return new error types ([#6]).
- `AsRef<Utf8Path>`, `AsRef<Path>`, `AsRef<str>` and `AsRef<OsStr>` impls for `Utf8Components`, `Utf8Component` and
  `Iter`.

[#6]: https://github.com/camino-rs/camino/issues/6

## [1.0.2] - 2021-03-02

### Added

- `From` impls for converting a `&Utf8Path` or a `Utf8PathBuf` into `Box<Path>`, `Rc<Path>`, `Arc<Path>` and `Cow<'a, Path>`.
- `PartialEq` and `PartialOrd` implementations comparing `Utf8Path` and `Utf8PathBuf` with `Path`, `PathBuf` and its
  variants, and comparing `OsStr`, `OsString` and its variants.

## [1.0.1] - 2021-02-25

### Added

- More `PartialEq` and `PartialOrd` implementations.
- MSRV lowered to 1.34.

## [1.0.0] - 2021-02-23

Initial release.

[1.0.9]: https://github.com/camino-rs/camino/releases/tag/camino-1.0.9
[1.0.8]: https://github.com/camino-rs/camino/releases/tag/camino-1.0.8
[1.0.7]: https://github.com/camino-rs/camino/releases/tag/camino-1.0.7
[1.0.6]: https://github.com/camino-rs/camino/releases/tag/camino-1.0.6
[1.0.5]: https://github.com/camino-rs/camino/releases/tag/camino-1.0.5
[1.0.4]: https://github.com/camino-rs/camino/releases/tag/camino-1.0.4
[1.0.3]: https://github.com/camino-rs/camino/releases/tag/camino-1.0.3
[1.0.2]: https://github.com/camino-rs/camino/releases/tag/camino-1.0.2
[1.0.1]: https://github.com/camino-rs/camino/releases/tag/camino-1.0.1
[1.0.0]: https://github.com/camino-rs/camino/releases/tag/camino-1.0.0
