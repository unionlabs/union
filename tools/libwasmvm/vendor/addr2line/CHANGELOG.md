## 0.19.0 (2022/11/24)

### Breaking changes

* Updated `gimli` and `object` dependencies.

--------------------------------------------------------------------------------

## 0.18.0 (2022/07/16)

### Breaking changes

* Updated `object` dependency.

### Changed

* Fixed handling of relative path for `DW_AT_comp_dir`.
  [#239](https://github.com/gimli-rs/addr2line/pull/239)

* Fixed handling of `DW_FORM_addrx` for DWARF 5 support.
  [#243](https://github.com/gimli-rs/addr2line/pull/243)

* Fixed handling of units that are missing range information.
  [#249](https://github.com/gimli-rs/addr2line/pull/249)

--------------------------------------------------------------------------------

## 0.17.0 (2021/10/24)

### Breaking changes

* Updated `gimli` and `object` dependencies.

### Changed

* Use `skip_attributes` to improve performance.
  [#236](https://github.com/gimli-rs/addr2line/pull/236)

--------------------------------------------------------------------------------

## 0.16.0 (2021/07/26)

### Breaking changes

* Updated `gimli` and `object` dependencies.

--------------------------------------------------------------------------------

## 0.15.2 (2021/06/04)

### Fixed

* Allow `Context` to be `Send`.
  [#219](https://github.com/gimli-rs/addr2line/pull/219)

--------------------------------------------------------------------------------

## 0.15.1 (2021/05/02)

### Fixed

* Don't ignore aranges with address 0.
  [#217](https://github.com/gimli-rs/addr2line/pull/217)

--------------------------------------------------------------------------------

## 0.15.0 (2021/05/02)

### Breaking changes

* Updated `gimli` and `object` dependencies.
  [#215](https://github.com/gimli-rs/addr2line/pull/215)

* Added `debug_aranges` parameter to `Context::from_sections`.
  [#200](https://github.com/gimli-rs/addr2line/pull/200)

### Added

* Added `.debug_aranges` support.
  [#200](https://github.com/gimli-rs/addr2line/pull/200)

* Added supplementary object file support.
  [#208](https://github.com/gimli-rs/addr2line/pull/208)

### Fixed

* Fixed handling of Windows paths in locations.
  [#209](https://github.com/gimli-rs/addr2line/pull/209)

* examples/addr2line: Flush stdout after each response.
  [#210](https://github.com/gimli-rs/addr2line/pull/210)

* examples/addr2line: Avoid copying every section.
  [#213](https://github.com/gimli-rs/addr2line/pull/213)

--------------------------------------------------------------------------------

## 0.14.1 (2020/12/31)

### Fixed

* Fix location lookup for skeleton units.
  [#201](https://github.com/gimli-rs/addr2line/pull/201)

### Added

* Added `Context::find_location_range`.
  [#196](https://github.com/gimli-rs/addr2line/pull/196)
  [#199](https://github.com/gimli-rs/addr2line/pull/199)

--------------------------------------------------------------------------------

## 0.14.0 (2020/10/27)

### Breaking changes

* Updated `gimli` and `object` dependencies.

### Fixed

* Handle units that only have line information.
  [#188](https://github.com/gimli-rs/addr2line/pull/188)

* Handle DWARF units with version <= 4 and no `DW_AT_name`.
  [#191](https://github.com/gimli-rs/addr2line/pull/191)

* Fix handling of `DW_FORM_ref_addr`.
  [#193](https://github.com/gimli-rs/addr2line/pull/193)

--------------------------------------------------------------------------------

## 0.13.0 (2020/07/07)

### Breaking changes

* Updated `gimli` and `object` dependencies.

* Added `rustc-dep-of-std` feature.
  [#166](https://github.com/gimli-rs/addr2line/pull/166)

### Changed

* Improve performance by parsing function contents lazily.
  [#178](https://github.com/gimli-rs/addr2line/pull/178)

* Don't skip `.debug_info` and `.debug_line` entries with a zero address.
  [#182](https://github.com/gimli-rs/addr2line/pull/182)

--------------------------------------------------------------------------------

## 0.12.2 (2020/06/21)

### Fixed

* Avoid linear search for `DW_FORM_ref_addr`.
  [#175](https://github.com/gimli-rs/addr2line/pull/175)

--------------------------------------------------------------------------------

## 0.12.1 (2020/05/19)

### Fixed

* Handle units with overlapping address ranges.
  [#163](https://github.com/gimli-rs/addr2line/pull/163)

* Don't assert for functions with overlapping address ranges.
  [#168](https://github.com/gimli-rs/addr2line/pull/168)

--------------------------------------------------------------------------------

## 0.12.0 (2020/05/12)

### Breaking changes

* Updated `gimli` and `object` dependencies.

* Added more optional features: `smallvec` and `fallible-iterator`.
  [#160](https://github.com/gimli-rs/addr2line/pull/160)

### Added

*  Added `Context::dwarf` and `Context::find_dwarf_unit`.
  [#159](https://github.com/gimli-rs/addr2line/pull/159)

### Changed

* Removed `lazycell` dependency.
  [#160](https://github.com/gimli-rs/addr2line/pull/160)

--------------------------------------------------------------------------------

## 0.11.0 (2020/01/11)

### Breaking changes

* Updated `gimli` and `object` dependencies.

* [#130](https://github.com/gimli-rs/addr2line/pull/130)
  Changed `Location::file` from `Option<String>` to `Option<&str>`.
  This required adding lifetime parameters to `Location` and other structs that
  contain it.

* [#152](https://github.com/gimli-rs/addr2line/pull/152)
  Changed `Location::line` and `Location::column` from `Option<u64>`to `Option<u32>`.

* [#156](https://github.com/gimli-rs/addr2line/pull/156)
  Deleted `alloc` feature, and fixed `no-std` builds with stable rust.
  Removed default `Reader` parameter for `Context`, and added `ObjectContext` instead.

### Added

* [#134](https://github.com/gimli-rs/addr2line/pull/134)
  Added `Context::from_dwarf`.

### Changed

* [#133](https://github.com/gimli-rs/addr2line/pull/133)
  Fixed handling of units that can't be parsed.

* [#155](https://github.com/gimli-rs/addr2line/pull/155)
  Fixed `addr2line` output to match binutils.

* [#130](https://github.com/gimli-rs/addr2line/pull/130)
  Improved `.debug_line` parsing performance.

* [#148](https://github.com/gimli-rs/addr2line/pull/148)
  [#150](https://github.com/gimli-rs/addr2line/pull/150)
  [#151](https://github.com/gimli-rs/addr2line/pull/151)
  [#152](https://github.com/gimli-rs/addr2line/pull/152)
  Improved `.debug_info` parsing performance.

* [#137](https://github.com/gimli-rs/addr2line/pull/137)
  [#138](https://github.com/gimli-rs/addr2line/pull/138)
  [#139](https://github.com/gimli-rs/addr2line/pull/139)
  [#140](https://github.com/gimli-rs/addr2line/pull/140)
  [#146](https://github.com/gimli-rs/addr2line/pull/146)
  Improved benchmarks.

--------------------------------------------------------------------------------

## 0.10.0 (2019/07/07)

### Breaking changes

* [#127](https://github.com/gimli-rs/addr2line/pull/127)
  Update `gimli`.

--------------------------------------------------------------------------------

## 0.9.0 (2019/05/02)

### Breaking changes

* [#121](https://github.com/gimli-rs/addr2line/pull/121)
  Update `gimli`, `object`, and `fallible-iterator` dependencies.

### Added

* [#121](https://github.com/gimli-rs/addr2line/pull/121)
  Reexport `gimli`, `object`, and `fallible-iterator`.

--------------------------------------------------------------------------------

## 0.8.0 (2019/02/06)

### Breaking changes

* [#107](https://github.com/gimli-rs/addr2line/pull/107)
  Update `object` dependency to 0.11. This is part of the public API.

### Added

* [#101](https://github.com/gimli-rs/addr2line/pull/101)
  Add `object` feature (enabled by default). Disable this feature to remove
  the `object` dependency and `Context::new` API.

* [#102](https://github.com/gimli-rs/addr2line/pull/102)
  Add `std` (enabled by default) and `alloc` features.

### Changed

* [#108](https://github.com/gimli-rs/addr2line/issues/108)
  `demangle` no longer ouputs the hash for rust symbols.

* [#109](https://github.com/gimli-rs/addr2line/issues/109)
  Set default `R` for `Context<R>`.
