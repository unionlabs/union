# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.3] - 2022-07-01

- Added support for thumbv6-none-eabi target.
- Fixed trap handling support on i686-windows targets.
- Fixed build on RISC-V targets.

## [0.1.2] - 2022-03-14

- Added the `on_stack` function which provides a simpler but faster alternative to coroutines when only stack switching is needed.
- Changed `on_parent_stack` to use `on_stack` internally.
- Fixed the RISC-V code to use the same frame pointer offset as GCC/LLVM.

## [0.1.1] - 2022-02-25

- Fixed image in README and crate documentation.
- Fixed RISC-V docs build on docs.rs.

## [0.1.0] - 2022-02-25

Initial release.

[unreleased]: https://github.com/Amanieu/corosensei/compare/v0.1.3...HEAD
[0.1.3]: https://github.com/Amanieu/corosensei/compare/v0.1.2...v0.1.3
[0.1.2]: https://github.com/Amanieu/corosensei/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/Amanieu/corosensei/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/Amanieu/corosensei/releases/tag/v0.1.0
