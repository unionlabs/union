# `wasm-bindgen` Change Log
--------------------------------------------------------------------------------

## 0.2.81

Released 2022-06-14.

[changes](https://github.com/rustwasm/wasm-bindgen/compare/0.2.80...0.2.81)

--------------------------------------------------------------------------------

## 0.2.80

Released 2022-04-04.

[changes](https://github.com/rustwasm/wasm-bindgen/compare/0.2.79...0.2.80)

--------------------------------------------------------------------------------

## 0.2.79

Released 2022-01-19.

[changes](https://github.com/rustwasm/wasm-bindgen/compare/0.2.78...0.2.79)

--------------------------------------------------------------------------------

## 0.2.78

Released 2021-09-15.

[changes](https://github.com/rustwasm/wasm-bindgen/compare/0.2.77...0.2.78)

--------------------------------------------------------------------------------

## 0.2.77

Released 2021-09-08.

[changes](https://github.com/rustwasm/wasm-bindgen/compare/0.2.76...0.2.77)

--------------------------------------------------------------------------------

## 0.2.76

Released 2021-08-19.

[changes](https://github.com/rustwasm/wasm-bindgen/compare/0.2.75...0.2.76)

--------------------------------------------------------------------------------

## 0.2.75

Released 2021-08-02.

[changes](https://github.com/rustwasm/wasm-bindgen/compare/0.2.74...0.2.75)

--------------------------------------------------------------------------------

## 0.2.74

Released 2021-05-10.

[changes](https://github.com/rustwasm/wasm-bindgen/compare/0.2.73...0.2.74)

--------------------------------------------------------------------------------

## 0.2.73

Released 2021-03-29.

[changes](https://github.com/rustwasm/wasm-bindgen/compare/0.2.72...0.2.73)

--------------------------------------------------------------------------------

## 0.2.72

Released 2021-03-18.

[changes](https://github.com/rustwasm/wasm-bindgen/compare/0.2.71...0.2.72)

--------------------------------------------------------------------------------

## 0.2.71

Released 2021-02-26.

[changes](https://github.com/rustwasm/wasm-bindgen/compare/0.2.70...0.2.71)

--------------------------------------------------------------------------------

## 0.2.70

Released 2021-01-25.

[changes](https://github.com/rustwasm/wasm-bindgen/compare/0.2.69...0.2.70)

--------------------------------------------------------------------------------

## 0.2.69

Released 2020-11-30.

### Added

* Unstable bindings for WebBluetooth have been added.
  [#2311](https://github.com/rustwasm/wasm-bindgen/pull/2311)

* Unstable bindings for WebUSB have been added.
  [#2345](https://github.com/rustwasm/wasm-bindgen/pull/2345)

* Renaming a struct field with `js_name` is now supported.
  [#2360](https://github.com/rustwasm/wasm-bindgen/pull/2360)

* The WebGPU WebIDL has been updated.
  [#2353](https://github.com/rustwasm/wasm-bindgen/pull/2353)

### Fixed

* The ImageCapture APIs of web-sys have been moved to unstable and were fixed.
  [#2348](https://github.com/rustwasm/wasm-bindgen/pull/2348)

* Bindings for `waitAsync` have been updated.
  [#2362](https://github.com/rustwasm/wasm-bindgen/pull/2362)

--------------------------------------------------------------------------------

## 0.2.68

Released 2020-09-08.

### Added

* Add userVisibleOnly property to PushSubscriptionOptionsInit.
  [#2288](https://github.com/rustwasm/wasm-bindgen/pull/2288)

### Fixed

* TypeScript files now import `*.wasm` instead of bare files.
  [#2283](https://github.com/rustwasm/wasm-bindgen/pull/2283)

* Usage of `externref` now appropriately resizes the table by using 2x the
  previous capacity, fixing a performance issue with lots of externref objects.
  [#2294](https://github.com/rustwasm/wasm-bindgen/pull/2294)

* Compatibility with the latest Firefox WebDriver has been fixed.
  [#2301](https://github.com/rustwasm/wasm-bindgen/pull/2301)

* Non deterministic output with closures has been fixed.
  [#2304](https://github.com/rustwasm/wasm-bindgen/pull/2304)

### Updated

* The WebGPU WebIDL was updated.
  [#2267](https://github.com/rustwasm/wasm-bindgen/pull/2267)

--------------------------------------------------------------------------------

## 0.2.67

Released 2020-07-28.

### Added

* A `--reference-types` flag was added to the CLI.
  [#2257](https://github.com/rustwasm/wasm-bindgen/pull/2257)

### Fixed

* Breakage with `Closure::forget` in 0.2.66 was fixed.
  [#2258](https://github.com/rustwasm/wasm-bindgen/pull/2258)

--------------------------------------------------------------------------------

## 0.2.66

Released 2020-07-28.

### Added

* Reverse mappings from value to name are now available in JS bindings of enums.
  [#2240](https://github.com/rustwasm/wasm-bindgen/pull/2240)

### Fixed

* Functions using a return pointer in threaded programs now correctly load and
  store return values in a way that doesn't interfere with other threads.
  [#2249](https://github.com/rustwasm/wasm-bindgen/pull/2249)

* Support for weak references has been updated and a `--weak-refs` flag is now
  available in the CLI for enabling weak references.
  [#2248](https://github.com/rustwasm/wasm-bindgen/pull/2248)

--------------------------------------------------------------------------------

## 0.2.65

Released 2020-07-15.

### Added

* Functions from JS can now be natively imported as `async` and will use
  promises under the hood.
  [#2196](https://github.com/rustwasm/wasm-bindgen/pull/2196)

### Changed

* Encoding for the reference types proposal has been updated to the latest
  version of the spec.
  [#2234](https://github.com/rustwasm/wasm-bindgen/pull/2234)

--------------------------------------------------------------------------------

## 0.2.64

Released 2020-06-29.

### Added

* Nested namespaces for imports can now be specified.
  [#2105](https://github.com/rustwasm/wasm-bindgen/pull/2105)

* A `deno` target has been added.
  [#2176](https://github.com/rustwasm/wasm-bindgen/pull/2176)

### Fixed

* Getters/setters that consume the original object have been fixed to invalidate
  the object correctly.
  [#2172](https://github.com/rustwasm/wasm-bindgen/pull/2172)

* Compatibility with nightly threading in LLVM has been fixed.
  [#2183](https://github.com/rustwasm/wasm-bindgen/pull/2183)

* Trailing space in generated doc comments is now removed.
  [#2210](https://github.com/rustwasm/wasm-bindgen/pull/2210)

--------------------------------------------------------------------------------

## 0.2.63

Released 2020-05-27.

### Added

* A new example about using WebRTC has been added.
  [#2131](https://github.com/rustwasm/wasm-bindgen/pull/2131)

* The `Blob.stream()` method has been added.
  [#2140](https://github.com/rustwasm/wasm-bindgen/pull/2140)
  [#2142](https://github.com/rustwasm/wasm-bindgen/pull/2142)

### Changed

* The encoding and implementation of WebAssembly reference types has been sync'd
  with the latest upstream specification.
  [#2125](https://github.com/rustwasm/wasm-bindgen/pull/2125)

### Fixed

* Test functions names will no longer collide with test intrinsic names.
  [#2123](https://github.com/rustwasm/wasm-bindgen/pull/2123)

* Fixed warnings with `#[must_use]` types in generated code.
  [#2144](https://github.com/rustwasm/wasm-bindgen/pull/2144)

* Fixed compatibility with latest Rust nightlies.
  [#2159](https://github.com/rustwasm/wasm-bindgen/pull/2159)

--------------------------------------------------------------------------------

## 0.2.62

Released 2020-05-01.

### Fixed

* Usage of `require` has been fixed with Webpack 5.
  [#2115](https://github.com/rustwasm/wasm-bindgen/pull/2115)

--------------------------------------------------------------------------------

## 0.2.61

Released 2020-04-29.

### Added

* Exported Rust `enum` types can now be renamed with `js_name`.
  [#2071](https://github.com/rustwasm/wasm-bindgen/pull/2071)

* More comments are copied to JS/TS files, and comments should no longer
  accidentally have escape sequences in them.
  [#2070](https://github.com/rustwasm/wasm-bindgen/pull/2070)

* Experimental bindings for the Clipboard browser APIs have been added.
  [#2100](https://github.com/rustwasm/wasm-bindgen/pull/2100)

### Changed

* WebGPU bindings have been updated.
  [#2080](https://github.com/rustwasm/wasm-bindgen/pull/2080)

* `setBindGroup` methods for WebIDL now take immutable slices instead of mutable
  slices.
  [#2087](https://github.com/rustwasm/wasm-bindgen/pull/2087)

* JS code generation for `catch` functions has been improved.
  [#2098](https://github.com/rustwasm/wasm-bindgen/pull/2098)

* Usage of NPM dependencies with the `web` target is no longer an error.
  [#2103](https://github.com/rustwasm/wasm-bindgen/pull/2103)

### Fixed

* Combining `js_name` with `getter` and `setter` has now been fixed.
  [#2074](https://github.com/rustwasm/wasm-bindgen/pull/2074)

* Importing global names which conflict with other namespaces should now work
  correctly.
  [#2057](https://github.com/rustwasm/wasm-bindgen/pull/2057)

* Acquiring the global JS object has been fixed for Firefox extension content
  scripts.
  [#2099](https://github.com/rustwasm/wasm-bindgen/pull/2099)

* The output of `wasm-bindgen` is now compatible with Webpack 5 and the updated
  version of the wasm ESM integration specification.
  [#2110](https://github.com/rustwasm/wasm-bindgen/pull/2099)

--------------------------------------------------------------------------------

## 0.2.60

Released 2020-03-25.

### Added

* The `js_sys` types are now more accurately reflected in TypeScript.
  [#2028](https://github.com/rustwasm/wasm-bindgen/pull/2028)

* The timeout in `wasm-bindgen-test-runner`'s timeout can now be configured via
  `WASM_BINDGEN_TEST_TIMEOUT`.
  [#2036](https://github.com/rustwasm/wasm-bindgen/pull/2036)

* WebIDL for WebXR has been added.
  [#2000](https://github.com/rustwasm/wasm-bindgen/pull/2000)

### Changed

* The WebIDL for WebGPU has been updated.
  [#2037](https://github.com/rustwasm/wasm-bindgen/pull/2037)

--------------------------------------------------------------------------------

## 0.2.59

Released 2020-03-03.

### Added

* The `js_sys::Number` type now has a number of JS-number associated constants
  on it now.
  [#1965](https://github.com/rustwasm/wasm-bindgen/pull/1965)

* The `getTransform` method on `CanvasRenderingContext2D` has been added.
  [#1966](https://github.com/rustwasm/wasm-bindgen/pull/1966)

* Initial experimental support was added for electron targets with a new
  `--omit-imports` flag.
  [#1958](https://github.com/rustwasm/wasm-bindgen/pull/1958)

* Optional struct fields are now reflected idiomatically in TypeScript.
  [#1990](https://github.com/rustwasm/wasm-bindgen/pull/1990)

* Typed arrays in `js_sys` now have `get_index` and `set_index` methods.
  [#2001](https://github.com/rustwasm/wasm-bindgen/pull/2001)

* The `web_sys::Blob` type has been updated with `arrayBuffer` and `text`
  methods.
  [#2008](https://github.com/rustwasm/wasm-bindgen/pull/2008)

* Support for unstable browser interfaces has now been added. By compiling
  `web_sys` with `--cfg web_sys_unstable_apis` (typically via `RUSTFLAGS`)
  you'll be able to access all bound WebIDL functions, even those like GPU
  support on the web, which has now also had its WebIDL updated.
  [#1997](https://github.com/rustwasm/wasm-bindgen/pull/1997)

* The compile time for `web_sys` has been massively reduced by pre-generating
  Rust code from WebIDL. It is also readable now since it generates
  `#[wasm_bindgen]` annotations instead of expanded code.
  [#2012](https://github.com/rustwasm/wasm-bindgen/pull/2012)

* A new `typescript_type` attribute can be used to specify the TypeScript type
  for an `extern` type. [#2012](https://github.com/rustwasm/wasm-bindgen/pull/2012)

* It is now possible to use string values with `#[wasm_bindgen]` `enum`s.
  [#2012](https://github.com/rustwasm/wasm-bindgen/pull/2012)

* A new `skip_tyepscript` attribute is recognized to skip generating TypeScript
  bindings for a function or type.
  [#2016](https://github.com/rustwasm/wasm-bindgen/pull/2016)

### Changed

* More `uniformMatrix*` bindings now are whitelisted take shared slice instead
  of a mutable slice.
  [#1957](https://github.com/rustwasm/wasm-bindgen/pull/1957)

* Non-`dependency` keys in `package.json` are now ignored instead of error'd
  about.
  [#1969](https://github.com/rustwasm/wasm-bindgen/pull/1969)

* WebGPU has been removed from `web_sys` since it was outdated and didn't work
  anywhere anyway.
  [#1972](https://github.com/rustwasm/wasm-bindgen/pull/1972)

* The JS heap of objects managed by wasm-bindgen has had its definition
  tightended up a bit.
  [#1987](https://github.com/rustwasm/wasm-bindgen/pull/1987)

* The `self` identifier is no longe used on the `no-modules` target, making it a
  bit more flexible in more environments.
  [#1995](https://github.com/rustwasm/wasm-bindgen/pull/1995)

* The wasm-loading logic is now more flexible and can take promises as well.
  [#1996](https://github.com/rustwasm/wasm-bindgen/pull/1996)

* JS glue for closures is now deduplicated.
  [#2002](https://github.com/rustwasm/wasm-bindgen/pull/2002)

* The `web_sys` crate now emits more accurate TypeScript definitions using named
  types instead of `any` everywhere.
  [#1998](https://github.com/rustwasm/wasm-bindgen/pull/1998)

* The `send_with_u8_array` methods in `web_sys` are whitelisted to take shared
  slices instead of mutable slices.
  [#2015](https://github.com/rustwasm/wasm-bindgen/pull/2015)

--------------------------------------------------------------------------------

## 0.2.58

Released 2020-01-07.

### Added

* When using the `no-modules` output type the initialization path for the wasm
  file is now optional if it can be inferred from the current JS script.
  [#1938](https://github.com/rustwasm/wasm-bindgen/pull/1938)

### Fixed

* TypeScript for struct methods that have floats has been fixed.
  [#1945](https://github.com/rustwasm/wasm-bindgen/pull/1945)

--------------------------------------------------------------------------------

## 0.2.57

Released 2020-01-06.

### Fixed

* The `js_sys::Promise` type is now marked as `#[must_use]`
  [#1927](https://github.com/rustwasm/wasm-bindgen/pull/1927)

* Duplicate imports of the same name are now handled correctly again.
  [#1942](https://github.com/rustwasm/wasm-bindgen/pull/1942)

--------------------------------------------------------------------------------

## 0.2.56

Released 2019-12-20.

### Added

* Added a `#[wasm_bindgen(inspectable)]` attribute for exported objects to
  generate `toJSON` and `toString` implementations.
  [#1876](https://github.com/rustwasm/wasm-bindgen/pull/1876)

* Support for the most recent interface types proposal has been implemented.
  [#1882](https://github.com/rustwasm/wasm-bindgen/pull/1882)

* Initial support for async iterators has been added.
  [#1895](https://github.com/rustwasm/wasm-bindgen/pull/1895)

* Support for an `async` start function was added.
  [#1905](https://github.com/rustwasm/wasm-bindgen/pull/1905)

* `Array::iter` and `Array::to_vec` methods were added to js-sys.
  [#1909](https://github.com/rustwasm/wasm-bindgen/pull/1909)

### Fixed

* Another webkit-specific WebIDL construct was fixed in web-sys.
  [#1865](https://github.com/rustwasm/wasm-bindgen/pull/1865)

--------------------------------------------------------------------------------

## 0.2.55

Released 2019-11-19.

### Fixed

* Running `wasm-bindgen` over empty anyref modules now works again.
  [#1861](https://github.com/rustwasm/wasm-bindgen/pull/1861)

* Support for multi-value JS engines has been fixed as a wasm interface types
  polyfill.
  [#1863](https://github.com/rustwasm/wasm-bindgen/pull/1863)

--------------------------------------------------------------------------------

## 0.2.54

Released 2019-11-07.

### Added

* A safe `to_vec` method has been added for typed arrays.
  [#1844](https://github.com/rustwasm/wasm-bindgen/pull/1844)

* A unsafe method `view_mut_raw` has been added to typed arrays.
  [#1850](https://github.com/rustwasm/wasm-bindgen/pull/1850)

* The `HTMLImageElement` WebIDL has been updated with recent features.
  [#1842](https://github.com/rustwasm/wasm-bindgen/pull/1842)

* Binary crates are now supported and `fn main` will be automatically executed
  like the `start` function.
  [#1843](https://github.com/rustwasm/wasm-bindgen/pull/1843)

### Changed

* Some JS glue generation has been tweaked to avoid TypeScript warnings.
  [#1852](https://github.com/rustwasm/wasm-bindgen/pull/1852)

--------------------------------------------------------------------------------

## 0.2.53

Released 2019-10-29.

### Fixed

* A bug with the experimental support for multi-value interface types has been
  fixed.
  [#1839](https://github.com/rustwasm/wasm-bindgen/pull/1839)

--------------------------------------------------------------------------------

## 0.2.52

Released 2019-10-24.

### Added

* The support for wasm-interface-types now uses multi-value by default.
  [#1805](https://github.com/rustwasm/wasm-bindgen/pull/1805)

* The Worklet IDL has been updated.
  [#1817](https://github.com/rustwasm/wasm-bindgen/pull/1817)

* The HTMLInputElement type has selectionStart and selectionEnd properties now.
  [#1811](https://github.com/rustwasm/wasm-bindgen/pull/1811)

* An `unintern` function has been added to remove an interned string from the
  cache.
  [#1828](https://github.com/rustwasm/wasm-bindgen/pull/1828)

### Changed

* Some WebIDL indexing getters have been corrected to reflect that they can
  throw and/or return `undefined`
  [#1789](https://github.com/rustwasm/wasm-bindgen/pull/1789)

### Fixed

* A bug with `TextDecoder` and Safari has been fxied
  [#1789](https://github.com/rustwasm/wasm-bindgen/pull/1789)

--------------------------------------------------------------------------------

## 0.2.51

Released 2019-09-26.

### Added

* The `wasm-bindgen-futures` and `wasm-bindgen-test` crates now require Nightly
  Rust and have a new major version published as a result. These crates now
  support `async`/`await` by default, and they will be supported in the stable
  Rust 1.39.0 release. The previous versions of crates will continue to work on
  stable today.
  [#1741](https://github.com/rustwasm/wasm-bindgen/pull/1741)

* Using `#[wasm_bindgen]` on an `async` function will now work and return a
  `Promise` on the JS side of things.
  [#1754](https://github.com/rustwasm/wasm-bindgen/pull/1754)

* More helper methods for `js_sys::Array` have been added.
  [#1749](https://github.com/rustwasm/wasm-bindgen/pull/1749)

* Initial support for the WebAssembly multi-value proposal has been added.
  [#1764](https://github.com/rustwasm/wasm-bindgen/pull/1764)

* Constructors for `js_sys::Date` with optional parameters has been added.
  [#1759](https://github.com/rustwasm/wasm-bindgen/pull/1759)

* Headless tests can now be run against a remote webdriver client
  [#1744](https://github.com/rustwasm/wasm-bindgen/pull/1744)

### Changed

* The `passStringToWasm` function has been optimized for size.
  [#1736](https://github.com/rustwasm/wasm-bindgen/pull/1736)

### Fixed

* BOM markers will not be preserved when passing strings to/from wasm.
  [#1730](https://github.com/rustwasm/wasm-bindgen/pull/1730)

* Importing a `static` value which isn't a `JsValue` has been fixed.
  [#1784](https://github.com/rustwasm/wasm-bindgen/pull/1784)

* Converting `undefined` to a Rust value via `into_serde` has been fixed.
  [#1783](https://github.com/rustwasm/wasm-bindgen/pull/1783)

* Routine errors are no longer erroneously logged in debug mode.
  [#1788](https://github.com/rustwasm/wasm-bindgen/pull/1788)

--------------------------------------------------------------------------------

## 0.2.50

Released 2019-08-19.

### Added

* Experimental support with a `WASM_INTERFACE_TYPES=1` environment variable has
  been added to emit a Wasm Interface Types custom section, making the output of
  `wasm-bindgen` a single standalone WebAssembly file.
  [#1725](https://github.com/rustwasm/wasm-bindgen/pull/1725)

### Fixed

* Unrelated errors are now no longer accidentally swallowed by the
  `instantiateStreaming` fallback.
  [#1723](https://github.com/rustwasm/wasm-bindgen/pull/1723)

--------------------------------------------------------------------------------

## 0.2.49

Released 2019-08-14.

### Added

* Add binding for `Element.getElementsByClassName`.
  [#1665](https://github.com/rustwasm/wasm-bindgen/pull/1665)

* `PartialEq` and `Eq` are now implementd for all `web-sys` types.
  [#1673](https://github.com/rustwasm/wasm-bindgen/pull/1673)

* The `wasm-bindgen-futures` crate now has support for futures when the
  experimental WebAssembly threading feature is enabled.
  [#1514](https://github.com/rustwasm/wasm-bindgen/pull/1514)

* A new `enable-interning` feature is available to intern strings and reduce the
  cost of transferring strings across the JS/Rust boundary.
  [#1612](https://github.com/rustwasm/wasm-bindgen/pull/1612)

* The `wasm-bindgen` CLI has experimental support for reading native
  `webidl-bindings` custom sections and generating JS glue. This support is in
  addition to Rust's own custom sections and allows using `wasm-bindgen` with
  binaries produced by other than rustc possibly.
  [#1690](https://github.com/rustwasm/wasm-bindgen/pull/1690)

* New environment variables have been added to configure webdriver startup
  arguments.
  [#1703](https://github.com/rustwasm/wasm-bindgen/pull/1703)

* New `JsValue::{is_truthy,is_falsy}` methods are now available.
  [#1638](https://github.com/rustwasm/wasm-bindgen/pull/1638)

### Changed

* JS import shims are now skipped again when they are unnecessary.
  [#1654](https://github.com/rustwasm/wasm-bindgen/pull/1654)

* WebAssembly output files now directly embed the module/name for imports if
  supported for the target and the import, reducing JS shims even further.
  [#1689](https://github.com/rustwasm/wasm-bindgen/pull/1689)

### Fixed

* Support for threads have been updated for LLVM 9 and nightly Rust.
  [#1675](https://github.com/rustwasm/wasm-bindgen/pull/1675)
  [#1688](https://github.com/rustwasm/wasm-bindgen/pull/1688)

* The `anyref` passes in `wasm-bindgen` have seen a number of fixes to improve
  their correctness and get the full test suite running.
  [#1692](https://github.com/rustwasm/wasm-bindgen/pull/1692)
  [#1704](https://github.com/rustwasm/wasm-bindgen/pull/1704)

* Support for `futures-preview 0.3.0-alpha.18` has been added to
  `wasm-bindgen-futures`.
  [#1716](https://github.com/rustwasm/wasm-bindgen/pull/1716)

--------------------------------------------------------------------------------

## 0.2.48

Released 2019-07-11.

### Added

* All typed arrays now implement `From` for the corresponding Rust slice type,
  providing a safe way to create an instance which copies the data.
  [#1620](https://github.com/rustwasm/wasm-bindgen/pull/1620)

* `Function::bind{2,3,4}` are now available in `js-sys`.
  [#1633](https://github.com/rustwasm/wasm-bindgen/pull/1633)

### Changed

* More WebGL methods have been updated to use shared slices instead of mutable
  slices.
  [#1639](https://github.com/rustwasm/wasm-bindgen/pull/1639)

* When using the `bundler` target the import of the wasm file now uses the
  `.wasm` extension to ensure a wasm file is loaded.
  [#1646](https://github.com/rustwasm/wasm-bindgen/pull/1646)

* The old internal `Stack` trait has been removed since it is no longer used.
  [#1624](https://github.com/rustwasm/wasm-bindgen/pull/1624)

### Fixed

* The `js_sys::global()` accessor now attempts other strategies before falling
  back to a `Function` constructor which can violate some strict CSP settings.
  [#1650](https://github.com/rustwasm/wasm-bindgen/pull/1649)

* Dropping a `JsFuture` no longer logs a benign error to the console.
  [#1649](https://github.com/rustwasm/wasm-bindgen/pull/1649)

* Fixed an assertion which could happen in some modules when generating
  bindings.
  [#1617](https://github.com/rustwasm/wasm-bindgen/pull/1617)

--------------------------------------------------------------------------------

## 0.2.47

Released 2019-06-19.

### Changed

* The `HtmlHyperlinkElement` should now include more native methods after a
  small edit to the WebIDL.
  [#1604](https://github.com/rustwasm/wasm-bindgen/pull/1604)

* Duplicate names for getters/setters now have a first-class `wasm-bindgen`
  error.
  [#1605](https://github.com/rustwasm/wasm-bindgen/pull/1605)

### Fixed

* TypeScript definition of `init` with `--target web` now reflects that the
  first argument is optional.
  [#1599](https://github.com/rustwasm/wasm-bindgen/pull/1599)

* A panic with the futures 0.3 support has been fixed.
  [#1598](https://github.com/rustwasm/wasm-bindgen/pull/1598)

* More slice types are recognized as becoming immutable in some WebIDL methods.
  [#1602](https://github.com/rustwasm/wasm-bindgen/pull/1602)

* The function table is now no longer too aggressively removed.
  [#1606](https://github.com/rustwasm/wasm-bindgen/pull/1606)

--------------------------------------------------------------------------------

## 0.2.46

Released 2019-06-14.

### Added

* Bindings for `Array#flat` and `Array#flatMap` have been added.
  [#1573](https://github.com/rustwasm/wasm-bindgen/pull/1573)

* All `#[wasm_bindgen]` types now `AsRef` to themslves.
  [#1583](https://github.com/rustwasm/wasm-bindgen/pull/1583)

* When using `--target web` the path passed to `init` is no longer required.
  [#1579](https://github.com/rustwasm/wasm-bindgen/pull/1579)

### Fixed

* Some diagnostics related to compiler errors in `#[wasm_bindgen]` have been
  improved.
  [#1550](https://github.com/rustwasm/wasm-bindgen/pull/1550)

* The support for weak references has been updated to the current JS proposal.
  [#1557](https://github.com/rustwasm/wasm-bindgen/pull/1557)

* Documentation and feature gating for web-sys dictionaries has improved.
  [#1572](https://github.com/rustwasm/wasm-bindgen/pull/1572)

* Getter and setter TypeScript has been fixed.
  [#1577](https://github.com/rustwasm/wasm-bindgen/pull/1577)

* The `env_logger` crate and its tree of dependencies is no longer required to
  build `web-sys`.
  [#1586](https://github.com/rustwasm/wasm-bindgen/pull/1586)

--------------------------------------------------------------------------------

## 0.2.45

Released 2019-05-20.

### Fixed

* Using `__wbindgen_cb_forget` on `--target web` has been fixed.
  [#1544](https://github.com/rustwasm/wasm-bindgen/pull/1544)

### Changed

* More whitelists have been added for `web-sys` to use shared slices instead of
  mutable slices.
  [#1539](https://github.com/rustwasm/wasm-bindgen/pull/1539)

--------------------------------------------------------------------------------

## 0.2.44

Released 2019-05-16.

### Added

* Support for exporting "fields" on JS objects wrapping Rust structs which are
  hooked up to getters/setters has been added. This is in addition to `pub`
  struct fields and allows performing more complicated computations in
  getters/setters.
  [#1440](https://github.com/rustwasm/wasm-bindgen/pull/1440)

* Support for futures 0.3 (and `async` / `await` syntax) has been added to the
  `wasm-bindgen-futures` crate.
  [#1507](https://github.com/rustwasm/wasm-bindgen/pull/1507)

* Stacks of imported JS functions that throw and aren't marked `catch` are now
  logged in debug mode.
  [#1466](https://github.com/rustwasm/wasm-bindgen/pull/1466)

* A utility for counting the size of the `anyref` heap has been added.
  [#1521](https://github.com/rustwasm/wasm-bindgen/pull/1521)

* Passing ASCII-only strings to WASM should now be significantly faster.
  [#1470](https://github.com/rustwasm/wasm-bindgen/pull/1470)

* The `selectionStart` and `selectionEnd` APIs of text areas have been enabled.
  [#1533](https://github.com/rustwasm/wasm-bindgen/pull/1533)

### Changed

* Some more methods in `web-sys` now take immutable slices instead of mutable
  ones.
  [#1508](https://github.com/rustwasm/wasm-bindgen/pull/1508)

* TypeScript bindings for `Option<T>` arguments now use `foo?` where possible.
  [#1483](https://github.com/rustwasm/wasm-bindgen/pull/1483)

### Fixed

* Unnecessary bindings to `__wbindgen_object_drop_ref` have been fixed.
  [#1504](https://github.com/rustwasm/wasm-bindgen/pull/1504)

* Some direct imports have been fixed for `--target web`.
  [#1503](https://github.com/rustwasm/wasm-bindgen/pull/1503)

* Both importing and exporting the same name has been fixed.
  [#1506](https://github.com/rustwasm/wasm-bindgen/pull/1506)

* TypeScript typings for `init` in `--target web` have been fixed.
  [#1520](https://github.com/rustwasm/wasm-bindgen/pull/1520)

* Calling a dropped `Closure` should no longer "segfault" but produce a clear
  error.
  [#1530](https://github.com/rustwasm/wasm-bindgen/pull/1530)

--------------------------------------------------------------------------------

## 0.2.43

Released 2019-04-29.

### Added

* Support for `isize` and `usize` arrays has been added.
  [#1448](https://github.com/rustwasm/wasm-bindgen/pull/1448)

* Support customizing `dyn_ref` and friends via a new `is_type_of` attribute and
  apply it to some `js_sys` bindings.
  [#1405](https://github.com/rustwasm/wasm-bindgen/pull/1405)
  [#1450](https://github.com/rustwasm/wasm-bindgen/pull/1450)
  [#1490](https://github.com/rustwasm/wasm-bindgen/pull/1490)

* A new `skip` attribute to `#[wasm_bindgen]` has been added to skip fields and
  methods when generating bindings.
  [#1410](https://github.com/rustwasm/wasm-bindgen/pull/1410)

* More bindings have been added to `web-sys` for interfaces tagged with
  `[NoInterfaceObject]` in WebIDL. These types always fail `dyn_ref` and friends
  and must be manually casted into.
  [#1449](https://github.com/rustwasm/wasm-bindgen/pull/1449)

* Added `Debug for JsFuture`.
  [#1477](https://github.com/rustwasm/wasm-bindgen/pull/1477)

* Initial bindings for `Atomics` and `SharedArrayBuffer` have been added to
  `js_sys`.
  [#1463](https://github.com/rustwasm/wasm-bindgen/pull/1463)

* Bindings for `Object.fromEntries` has been added to `js_sys`.
  [#1456](https://github.com/rustwasm/wasm-bindgen/pull/1456)

* Tuple structs exported to JS now have indexed struct properties.
  [#1467](https://github.com/rustwasm/wasm-bindgen/pull/1467)

* Binding for `new Function(args, body)` has been added to `js_sys`.
  [#1492](https://github.com/rustwasm/wasm-bindgen/pull/1492)

* Bindings for some variadic functions have been added to `js_sys`.
  [#1491](https://github.com/rustwasm/wasm-bindgen/pull/1491)

### Changed

* Many `js-sys` types have received various tweaks and improvements to ensure
  they're consistent and work similarly to native Rust types.
  [#1447](https://github.com/rustwasm/wasm-bindgen/pull/1447)
  [#1444](https://github.com/rustwasm/wasm-bindgen/pull/1444)
  [#1473](https://github.com/rustwasm/wasm-bindgen/pull/1473)

* Dummy types in `js-sys` only used to namespace methods were removed and now
  modules are used for namespacing instead.
  [#1451](https://github.com/rustwasm/wasm-bindgen/pull/1451)

* Bindings in `web-sys` are formatted by default for ease of usage in IDEs.
  [#1461](https://github.com/rustwasm/wasm-bindgen/pull/1461)

### Fixed

* Documentation for Rust methods now show up in TypeScript as well.
  [#1472](https://github.com/rustwasm/wasm-bindgen/pull/1472)

--------------------------------------------------------------------------------

## 0.2.42

Released 2019-04-11.

### Fixed

* Fixed an issue in Firefox where using `encodeInto` accidentally caused empty
  strings to keep getting passed to Rust.
  [#1434](https://github.com/rustwasm/wasm-bindgen/pull/1434)

--------------------------------------------------------------------------------

## 0.2.41

Released 2019-04-10.

### Added

* Initial support for transitive NPM dependencies has been added.
  [#1305](https://github.com/rustwasm/wasm-bindgen/pull/1305)

* The `constructor` property of `Object` is now bound in `js-sys`.
  [#1403](https://github.com/rustwasm/wasm-bindgen/pull/1403)

* The `Closure` type now always implements `Debug`.
  [#1408](https://github.com/rustwasm/wasm-bindgen/pull/1408)

* Closures which take one `&T` argument are now supported. More implementations
  may be added in the future, but for now it's just one argument closures.
  [#1417](https://github.com/rustwasm/wasm-bindgen/pull/1417)

* The TypeScript bindings for `--web` now expose the `init` function.
  [#1412](https://github.com/rustwasm/wasm-bindgen/pull/1412)

* A `js_sys::JsString::is_valid_utf16` method has been added to handle unpaired
  surrogates in JS strings. Surrounding documentation has also been updated to
  document this potential pitfall.
  [#1416](https://github.com/rustwasm/wasm-bindgen/pull/1416)

* A `wasm_bindgen::function_table()` function has been added to expose the
  `WebAssembly.Table` and get access to it in wasm code.
  [#1431](https://github.com/rustwasm/wasm-bindgen/pull/1431)

### Fixed

* Reexporting the `wasm_bindgen` macro in crates has been fixed.
  [#1359](https://github.com/rustwasm/wasm-bindgen/pull/1359)

* Returning `u32` to JS has been fixed where large `u32` values would show up in
  JS as large negative numbers.
  [#1401](https://github.com/rustwasm/wasm-bindgen/pull/1401)

* Manual instantiation with `WebAssembly.Module` has been fixed.
  [#1419](https://github.com/rustwasm/wasm-bindgen/pull/1419)

* Error message for non-`Copy` public struct fields has been improved.
  [#1430](https://github.com/rustwasm/wasm-bindgen/pull/1430)

### Changed

* Performance of passing strings to Rust in Node.js has been improved.
  [#1391](https://github.com/rustwasm/wasm-bindgen/pull/1391)

* Performance of `js_sys::try_iter` has been improved.
  [#1393](https://github.com/rustwasm/wasm-bindgen/pull/1393)

* Performance of using `TextEncoder#encodeInto` has been improved.
  [#1414](https://github.com/rustwasm/wasm-bindgen/pull/1414)

--------------------------------------------------------------------------------

## 0.2.40

Released 2019-03-21.

### Added

* TypeScript and JS generation will now attempt to preserve argument names in
  the generated JS where possible.
  [#1344](https://github.com/rustwasm/wasm-bindgen/pull/1344)

* Enable `Option<T>` support for enums defined in WebIDL.
  [#1350](https://github.com/rustwasm/wasm-bindgen/pull/1350)

* Add a `raw_module` attribute to `#[wasm_bindgen]` which is the same as
  `module` except doesn't attempt to recognize `./`, `../`, `or `/` prefixed
  paths.
  [#1353](https://github.com/rustwasm/wasm-bindgen/pull/1353)

* The `wasm-bindgen` CLI flags have now all been renamed under a `--target`
  flag. Instead of `--web` you'll now pass `--target web`, for example. This
  increases consistency between the `wasm-bindgen` and `wasm-pack` CLI.
  [#1369](https://github.com/rustwasm/wasm-bindgen/pull/1369)

### Fixed

* Definitions for `TypedArray` imports of `js-sys` have been unified with a
  macro to improve consistency and fix future bugs.
  [#1371](https://github.com/rustwasm/wasm-bindgen/pull/1371)

* Usage of `--no-modules` in CloudFlare workers should now work by default.
  [#1384](https://github.com/rustwasm/wasm-bindgen/pull/1384)

* A use-after-free when a closure is reinvoked after being destroyed on the Rust
  die has been fixed.
  [#1385](https://github.com/rustwasm/wasm-bindgen/pull/1385)

* A bug causing nondeterministic generation of JS bindings has been fixed.
  [#1383](https://github.com/rustwasm/wasm-bindgen/pull/1383)

--------------------------------------------------------------------------------

## 0.2.39

Released 2018-03-13.

### Added

* Crates can now import locally written JS snippets to get bundled into the
  final output. See [RFC 6] for more details as well as the PR.
  [#1295](https://github.com/rustwasm/wasm-bindgen/pull/1295)

[RFC 6]: https://github.com/rustwasm/rfcs/pull/6

### Changed

* A typo in the return value of `slice` methods on typed arrays in `js-sys` was
  corrected.
  [#1321](https://github.com/rustwasm/wasm-bindgen/pull/1321)

* The directory specified by `--out-dir` is now created if it doesn't exist
  already.
  [#1330](https://github.com/rustwasm/wasm-bindgen/pull/1330)

### Fixed

* A bug where if `nom` was in a crate graph and was compiled with the
  `verbose-errors` feature has been fixed. Previously the `wasm-bindgen-webidl`
  crate wouldn't compile, and now it will.
  [#1338](https://github.com/rustwasm/wasm-bindgen/pull/1338)

--------------------------------------------------------------------------------

## 0.2.38

Released 2019-03-04.

### Added

* Support for `Option<RustStruct>` in `#[wasm_bindgen]` functions has now been
  added.
  [#1275](https://github.com/rustwasm/wasm-bindgen/pull/1275)

* Experimental support for the `anyref` type proposal in WebAssembly has now
  landed and is enabled with `WASM_BINDGEN_ANYREF=1`.
  [#1002](https://github.com/rustwasm/wasm-bindgen/pull/1002)

* Support fot the new browser `TextEncode#encodeInto` API has been added.
  [#1279](https://github.com/rustwasm/wasm-bindgen/pull/1279)

* JS doc comments are now added to TypeScript bindings in addition to the JS
  bindings generated.
  [#1302](https://github.com/rustwasm/wasm-bindgen/pull/1302)

* Initial support for `FnOnce` closures has been added to the `Closure` type.
  [#1281](https://github.com/rustwasm/wasm-bindgen/pull/1281)

### Fixed

* Fixed an internal assert tripping when some modules were compiled with LTO.
  [#1274](https://github.com/rustwasm/wasm-bindgen/pull/1274)

* The `Context` type in the `wasm-bindgen-test` crate had its JS name changed to
  avoid conflicts with other crates that have a `Context` type being exported.
  [#1280](https://github.com/rustwasm/wasm-bindgen/pull/1280)

* The headless test runner for Safari on macOS High Sierra has been fixed.
  [#1298](https://github.com/rustwasm/wasm-bindgen/pull/1298)

### Changed

* The `wasm-bindgen` CLI tool now emits the `producers` section again with
  relevant bugs having been fixed in the meantime. The
  `--remove-producers-section` flag can continue to be used to omit emission of
  this section.
  [#1263](https://github.com/rustwasm/wasm-bindgen/pull/1263)

--------------------------------------------------------------------------------

## 0.2.37

Released 2019-02-15.

### Added

* The `HtmlMediaElement` type now exposes a `src_object` getter.
  [#1248](https://github.com/rustwasm/wasm-bindgen/pull/1248).

* The `js_sys::Reflect` type now has specializes getter/setters for `u32` and
  `f64` indices.
  [#1225](https://github.com/rustwasm/wasm-bindgen/pull/1225).

* A `--remove-producers-section` flag has been added to the CLI tool to, well,
  remove the `producers` section from the final wasm file.
  [#1256](https://github.com/rustwasm/wasm-bindgen/pull/1256).

### Fixed

* The `wasm-bindgen` CLI tool will correctly strip DWARF debug information
  unless `--keep-debug` is passed.
  [#1255](https://github.com/rustwasm/wasm-bindgen/pull/1255).

### Changed

* The `wasm-bindgen` CLI tool no longer emits the `producers` custom section by
  default to work around a [webpack bug]. See
  [#1260](https://github.com/rustwasm/wasm-bindgen/pull/1260).

[webpack bug]: https://github.com/webpack/webpack/pull/8786

--------------------------------------------------------------------------------

## 0.2.36

Released 2019-02-12.

### Fixed

* Fixed a bug where using closures and LTO together caused a panic inside the
  `wasm-bindgen` CLI tool. See
  [#1244](https://github.com/rustwasm/wasm-bindgen/issues/1244).

--------------------------------------------------------------------------------

## 0.2.35

Released 2019-02-12.

### Changed

* `wasm-bindgen` now internally uses the `walrus` crate to perform its
  transformations of the wasm that rustc/LLVM emits. See
  [#1237](https://github.com/rustwasm/wasm-bindgen/pull/1237).

### Fixed

* When `WebAssembly.instantiateStreaming` fails due to incorrect MIME type,
  *actually* properly recover. See
  [#1243](https://github.com/rustwasm/wasm-bindgen/pull/1243).

--------------------------------------------------------------------------------

## 0.2.34

Released 2019-02-11.

### Added

* Added support for optional `enum`s. See
  [#1214](https://github.com/rustwasm/wasm-bindgen/pull/1214).
* Added the `UnwrapThrowExt<T>` trait, which can enable smaller code sizes for
  panics. See [#1219](https://github.com/rustwasm/wasm-bindgen/pull/1219).

### Fixed

* Some `WebGlRenderingContext` methods are now whitelisted to use shared slices
  instead of exclusive slices. See
  [#1199](https://github.com/rustwasm/wasm-bindgen/pull/1199).
* Fixed TypeScript definitions for optional types. See
  [#1201](https://github.com/rustwasm/wasm-bindgen/pull/1201).
* Quiet clippy warnings inside generated code. See
  [1207](https://github.com/rustwasm/wasm-bindgen/pull/1207).
* Fixed using `cfg_attr` and `wasm_bindgen` together like `#[cfg_attr(...,
  wasm_bindgen)]`. See
  [1208](https://github.com/rustwasm/wasm-bindgen/pull/1208).
* The WebAudio example program was fixed. See
  [#1215](https://github.com/rustwasm/wasm-bindgen/pull/1215).
* Fixed logging HTML in `wasm-bindgen-test`. See
  [#1233](https://github.com/rustwasm/wasm-bindgen/pull/1233).
* When `WebAssembly.instantiateStreaming` fails due to incorrect MIME type,
  properly recover. See
  [#1235](https://github.com/rustwasm/wasm-bindgen/pull/1235).

--------------------------------------------------------------------------------

## 0.2.33

Released 2019-01-18.

### Added

* Improved the `Debug` output of `JsValue`
  [#1161](https://github.com/rustwasm/wasm-bindgen/pull/1161)

* Bindings for `JSON.stringify` and its optional arguments have been added
  [#1190](https://github.com/rustwasm/wasm-bindgen/pull/1190)

### Fixed

* A bug with windows binaries being released has ben resolved.

--------------------------------------------------------------------------------

## 0.2.32

Released 2019-01-16.

### Added

* Added support for Web IDL sequences. This enabled bindings generation for a
  couple more Web APIs. We generate functions for Web APIs that take sequences
  to accept any iterable, and for Web APIs that return sequences, a
  `js_sys::Array` is returned. See
  [#1152](https://github.com/rustwasm/wasm-bindgen/pull/1152) and
  [#1038](https://github.com/rustwasm/wasm-bindgen/issues/1038).
* The `wasm-bindgen-test` test runner will capture `console.debug`,
  `console.info`, and `console.warn` log messages and print them to `stdout`
  now. It already supported `console.log` and `console.error` and continues to
  support them. See
  [#1183](https://github.com/rustwasm/wasm-bindgen/issues/1183) and
  [#1184](https://github.com/rustwasm/wasm-bindgen/pull/1184).
* Added additional `--debug`-only assertions in the emitted JS glue for cases
  where an imported JS function that is not annotated with
  `#[wasm_bindgen(catch)]` throws an exception. This should help catch some bugs
  earlier! See [#1179](https://github.com/rustwasm/wasm-bindgen/pull/1179).

### Fixed

* Fixed a bug where `#[wasm_bindgen_test]` tests would fail in non-headless Web
  browsers if they used `console.log`. See
  [#1167](https://github.com/rustwasm/wasm-bindgen/pull/1167).
* Fixed a bug where returning closures from exported functions sometimes
  resulted in a faulty error. See
  [#1174](https://github.com/rustwasm/wasm-bindgen/issues/1174) and
  [#1175](https://github.com/rustwasm/wasm-bindgen/pull/1175).
* Sometimes our generated TypeScript interface files had syntax errors in them
  (missing semicolons). This has been fixed. See
  [#1181](https://github.com/rustwasm/wasm-bindgen/pull/1181).

--------------------------------------------------------------------------------

## 0.2.31

Released 2019-01-09.

### Added

* A new `spawn_local` function has been added to the `wasm-bindgen-futures`
  crate.
  [#1148](https://github.com/rustwasm/wasm-bindgen/pull/1148)

* Built-in conversions are now available from typed arrays and Rust arrays.
  [#1147](https://github.com/rustwasm/wasm-bindgen/pull/1147)

### Fixed

* Some casing of dictionary properties in WebIDL has been fixed.
  [#1155](https://github.com/rustwasm/wasm-bindgen/pull/1155)

--------------------------------------------------------------------------------

## 0.2.30

Released 2019-01-07.

### Added

* The `wasm-bindgen` CLI now has an `--out-name` argument to name the output
  module.
  [#1084](https://github.com/rustwasm/wasm-bindgen/pull/1084)

* Support for importing the `default` export has been added.
  [#1106](https://github.com/rustwasm/wasm-bindgen/pull/1106)

### Changed

* All `web-sys` methods are now flagged as `structural`, fixing a few bindings.
  [#1117](https://github.com/rustwasm/wasm-bindgen/pull/1117)

### Fixed

* A small bug with LTO and closures has been fixed.
  [#1145](https://github.com/rustwasm/wasm-bindgen/pull/1145)

--------------------------------------------------------------------------------

## 0.2.29

Released 2018-12-04.

### Added

* Add a `#[wasm_bindgen(start)]` attribute to customize the `start` section of
  the wasm module.
  [#1057](https://github.com/rustwasm/wasm-bindgen/pull/1057)

* Add support for producing the new "producers" section of wasm binaries
  [#1041](https://github.com/rustwasm/wasm-bindgen/pull/1041)

* Add support a `typescript_custom_section` attribute for producing custom
  typescript abstractions
  [#1048](https://github.com/rustwasm/wasm-bindgen/pull/1048)

* Generate `*.d.ts` files for wasm files in addition to the JS bindings
  [#1053](https://github.com/rustwasm/wasm-bindgen/pull/1053)

* Add a feature to assert that all attributes in `#[wasm_bindgen]` are used to
  help catch typos and mistakes
  [#1055](https://github.com/rustwasm/wasm-bindgen/pull/1055)

### Changed

* JS glue generation has received a few small optimizations such as removing
  shims and removing object allocations
  [#1033](https://github.com/rustwasm/wasm-bindgen/pull/1033)
  [#1030](https://github.com/rustwasm/wasm-bindgen/pull/1030)

* JS glue now just uses one array of JS objects instead of two
  [#1069](https://github.com/rustwasm/wasm-bindgen/pull/1069)

### Fixed

* Fix a typo in the `--no-modules` generated JS
  [#1045](https://github.com/rustwasm/wasm-bindgen/pull/1045)

--------------------------------------------------------------------------------

## 0.2.28

Released 2018-11-12.

### Added

* The `js_class` support is now supported on exported types to define a
  different class in JS than is named in Rust
  [#1012](https://github.com/rustwasm/wasm-bindgen/pull/1012)

* More WebIDL bindings are exposed with some internal restructuring to ignore
  unimplemented types at a different location
  [#1014](https://github.com/rustwasm/wasm-bindgen/pull/1014)

* All imported types now implement `Deref` to their first `extends` attribute
  (or `JsValue` if one isn't listed). This is intended to greatly improve the
  ergonomics of `web-sys` bindings by allowing easy access to parent class
  methods
  [#1019](https://github.com/rustwasm/wasm-bindgen/pull/1019)

* A new attribute, `final`, can be applied to JS imports. This attribute is
  relatively nuanced and [best explained in documentation][final-dox], but is
  added since `structural` is now the default
  [#1019](https://github.com/rustwasm/wasm-bindgen/pull/1019)

[final-dox]: https://rustwasm.github.io/wasm-bindgen/reference/attributes/on-js-imports/final.html

* A new CLI flag, `--remove-name-section`, can be passed to remove the wasm
  `name` section which contains the names of functions for debugging (typically
  not needed in release mode)
  [#1024](https://github.com/rustwasm/wasm-bindgen/pull/1024)

### Changed

* All imported functions are now `structural` by default. This shouldn't change
  the semantics of imported functions, only how they're invoked in the JS
  function shims that are generated by `wasm-bindgen`. More discussion can be
  founed on [RFC 5] and the PR
  [#1019](https://github.com/rustwasm/wasm-bindgen/pull/1019)

[RFC 5]: https://rustwasm.github.io/rfcs/005-structural-and-deref.html

* JS glue assertions for moved arguments are now only emitted in debug mode,
  which is still off by default
  [#1020](https://github.com/rustwasm/wasm-bindgen/pull/1020)

### Fixed

* Typescript generated bindings now correctly reflect `Option<T>` for more types
  [#1008](https://github.com/rustwasm/wasm-bindgen/pull/1008)

* The JS shim code generation has been optimized for `structural` bindings (now
  the default) to include fewer JS shims and more easily optimizable for JS
  engines
  [#1019](https://github.com/rustwasm/wasm-bindgen/pull/1019)

* Passing a `WebAssembly.Module` to the `--no-modules` constructor has been
  fixed
  [#1025](https://github.com/rustwasm/wasm-bindgen/pull/1025)

--------------------------------------------------------------------------------

## 0.2.27

Released 2018-10-29.

### Fixed

* Fixed an internal panic where the gc passes were being too aggressive
  [#995](https://github.com/rustwasm/wasm-bindgen/pull/995)

--------------------------------------------------------------------------------

## 0.2.26

Released 2018-10-29.

### Added

* The `TypedArray.slice` methods have now been bound in `js-sys`.
  [#956](https://github.com/rustwasm/wasm-bindgen/pull/956)

* The `Debug` and `Clone` traits are now implemented for `js_sys::Promise`.
  [#957](https://github.com/rustwasm/wasm-bindgen/pull/957)

* The `js_sys::DataView` type now exposes overloads to specify endianness.
  [#966](https://github.com/rustwasm/wasm-bindgen/pull/966)

* When using `--no-modules` a `WebAssembly.Module` can now be directly passed
  into the instantiation glue.
  [#969](https://github.com/rustwasm/wasm-bindgen/pull/969)

### Fixed

* The `JsValue` type is no longer considered `Send`.
  [#955](https://github.com/rustwasm/wasm-bindgen/pull/955)

* The generated JS glue is now more robust in the face of missing APIs.
  [#959](https://github.com/rustwasm/wasm-bindgen/pull/959)

* An issue with the latest version of `safaridriver` used to run headless tests
  has been resolved.
  [#991](https://github.com/rustwasm/wasm-bindgen/pull/991)

--------------------------------------------------------------------------------

## 0.2.25

Released 2018-10-10.

### Fixed

* Using `wasm-bindgen` will no longer unconditionally pull in Rust's default
  allocator for Wasm (dlmalloc) regardless if you configured a custom global
  allocator (eg wee_alloc).
  [#947](https://github.com/rustwasm/wasm-bindgen/pull/947)

* Fixed web-sys build on some Windows machines.
  [#943](https://github.com/rustwasm/wasm-bindgen/issues/943)

* Fixed generated ES class bindings to Rust structs that were only referenced
  through struct fields.
  [#948](https://github.com/rustwasm/wasm-bindgen/issues/948)

--------------------------------------------------------------------------------

## 0.2.24

Released 2018-10-05.

### Added

* Constructors for types in `web-sys` should now have better documentation.

* A new `vendor_prefix` attribute in `#[wasm_bindgen]` is supported to bind APIs
  on the web which may have a vendor prefix (like `webkitAudioContext`). This is
  then subsequently used to fix `AudioContext` usage in Safari.

* The `#[wasm_bindgen(extends = Foo)]` attribute now supports full paths, so you
  can also say `#[wasm_bindgen(extends = foo::Bar)]` and such.

### Changed

* The `Closure<T>` type is now optimized when the underlying closure is a ZST.
  The type now no longer allocates memory in this situation.

* The documentation now has a list of caveats for browser support, including how
  `TextEncoder` and `TextDecoder` are not implemented in Edge. If you're using
  webpack there's a listed strategy available, and improvements to the polyfill
  strategy are always welcome!

* The `BaseAudioContext` and `AudioScheduledSourceNode` types in `web-sys` have
  been deprecated as they don't exist in Safari or Edge.

### Fixed

* Fixed the `#[wasm_bindgen_test]`'s error messages in a browser to correctly
  escape HTML-looking output.

* WebIDL Attributes on `Window` are now correctly bound to not go through
  `Window.prototype` which doesn't exist but instead use a `structural`
  definition.

* Fixed a codegen error when the `BorrowMut` trait was in scope.

* Fixed TypeScript generation for constructors of classes, it was accidentally
  producing a syntactially invalid file!

--------------------------------------------------------------------------------

## 0.2.23

Released 2018-09-26.

### Added

* [This is the first release of the `web-sys`
  crate!](https://rustwasm.github.io/2018/09/26/announcing-web-sys.html)

* Added support for unions of interfaces and non-interfaces in the WebIDL
  frontend.

* Added a policy for inclusion of new ECMAScript features in `js-sys`: the
  feature must be in stage 4 or greater for us to support it.

* Added some documentation about size profiling and optimization with
  `wasm-bindgen` to the guide.

* Added the `Clamped<T>` type for generating JavaScript `Uint8ClampedArray`s.

* CI is now running on beta! Can't wait for the `rustc` release trains to roll
  over, so we can run CI on stable too!

* Added the `js_sys::try_iter` function, which checks arbitrary JS values for
  compliance with the JS iteration protocol, and if they are iterable, converts
  them into an iterator over the JS values that they yield.

### Changed

* We now only generate null checks on methods on the JS side when in debug
  mode. For safety we will always null check on the Rust side, however.

* Improved error messages when defining setters that don't start with `set_` and
  don't use `js_name = ...`.

* Improved generated code for classes in a way that avoids an unnecessary
  allocation with static methods that return `Self` but are not the "main"
  constructor.

* **BREAKING:** `js_sys::Reflect` APIs are all fallible now. This is because
  reflecting on `Proxy`s whose trap handlers throw an exception can cause any of
  the reflection APIs to throw. Accordingly, `js_sys` has been bumped from
  `0.2.X` to `0.3.X`.

### Fixed

* The method of ensuring that `__wbindgen_malloc` and `__wbindgen_free` are
  always emitted in the `.wasm` binary, regardless of seeming reachability is
  now zero-overhead.

--------------------------------------------------------------------------------

## 0.2.22

Released 2018-09-21

### Added

* The `IntoIterator` trait is now implemented for JS `Iterator` types
* A number of variadic methods in `js-sys` have had explicit arities added.
* The guide has been improved quite a bit as well as enhanced with more examples
* The `js-sys` crate is now complete! Thanks so much to everyone involved to
  help fill out all the APIs.
* Exported Rust functions with `#[wasm_bindgen]` can now return a `Result` where
  the `Err` payload is raised as an exception in JS.

### Fixed

* An issue with running `wasm-bindgen` on crates that have been compiled with
  LTO has been resolved.

--------------------------------------------------------------------------------

## 0.2.21

Released 2018-09-07

### Added

* Added many more bindings for `WebAssembly` in the `js-sys` crate.

### Fixed

* The "names" section of the wasm binary is now correctly preserved by
  wasm-bindgen.

--------------------------------------------------------------------------------

## 0.2.20

Released 2018-09-06

### Added

* All of `wasm-bindgen` is configured to compile on stable Rust as of the
  upcoming 1.30.0 release, scheduled for October 25, 2018.
* The underlying `JsValue` of a `Closure<T>` type can now be extracted at any
  time.
* Initial and experimental support was added for modules that have shared memory
  (use atomic instructions).

### Removed

* The `--wasm2asm` flag of `wasm2es6js` was removed because the `wasm2asm` tool
  has been removed from upstream Binaryen. This is replaced with the new
  `wasm2js` tool from Binaryen.

### Fixed

* The "schema" version for wasm-bindgen now changes on all publishes, meaning we
  can't forget to update it. This means that the crate version and CLI version
  must exactly match.
* The `wasm-bindgen` crate now has a `links` key which forbids multiple versions
  of `wasm-bindgen` from being linked into a dependency graph, fixing obscure
  linking errors with a more first-class error message.
* Binary releases for Windows has been fixed.

--------------------------------------------------------------------------------

## 0.2.19 (and 0.2.18)

Released 2018-08-27.

### Added

* Added bindings to `js-sys` for some `WebAssembly` types.
* Added bindings to `js-sys` for some `Intl` types.
* Added bindings to `js-sys` for some `String` methods.
* Added an example of using the WebAudio APIs.
* Added an example of using the `fetch` API.
* Added more `extends` annotations for types in `js-sys`.
* Experimental support for `WeakRef` was added to automatically deallocate Rust
  objects when gc'd.
* Added support for executing `wasm-bindgen` over modules that import their
  memory.
* Added a global `memory()` function in the `wasm-bindgen` crate for accessing
  the JS object that represent wasm's own memory.

### Removed

* Removed `AsMut` implementations for imported objects.

### Fixed

* Fixed the `constructor` and `catch` attributes combined on imported types.
* Fixed importing the same-named static in two modules.

--------------------------------------------------------------------------------

## 0.2.17

Released 2018-08-16.

### Added

* Greatly expanded documentation in the wasm-bindgen guide.
* Added bindings to `js-sys` for `Intl.DateTimeFormat`
* Added a number of `extends` attributes for types in `js-sys`

### Fixed

* Fixed compile on latest nightly with latest `proc-macro2`
* Fixed compilation in some scenarios on Windows with paths in `module` paths

--------------------------------------------------------------------------------

## 0.2.16

Released 2018-08-13.

### Added

* Added the `wasm_bindgen::JsCast` trait, as described in [RFC #2][rfc-2].
* Added [the `#[wasm_bindgen(extends = ...)]` attribute][extends-attr] to
  describe inheritance relationships, as described in [RFC #2][rfc-2].
* Added support for receiving `Option<&T>` parameters from JavaScript in
  exported Rust functions and methods.
* Added support for receiving `Option<u32>` and other option-wrapped scalars.
* Added reference documentation to the guide for every `#[wasm_bindgen]`
  attribute and how it affects the generated bindings.
* Published the `wasm-bindgen-futures` crate for converting between JS
  `Promise`s and Rust `Future`s.

### Changed

* Overhauled the guide's documentation on passing JS closures to Rust, and Rust
  closures to JS.
* Overhauled the guide's documentation on using serde to serialize complex data
  to `JsValue` and deserialize `JsValue`s back into complex data.
* Static methods are now always bound to their JS class, as is required for
  `Promise`'s static methods.

### Removed

* Removed internal usage of `syn`'s `visit-mut` cargo feature, which should
  result in faster build times.

### Fixed

* Various usage errors for the `#[wasm_bindgen]` proc-macro are now properly
  reported with source span information, rather than `panic!()`s inside the
  proc-macro.
* Fixed a bug where taking a struct by reference and returning a slice resulted
  in lexical variable redeclaration errors in the generated JS glue. [#662][]
* The `#[wasm_bindgen(js_class = "....")]` attribute for binding methods to
  renamed imported JS classes now properly works with constructors.

[rfc-2]: https://rustwasm.github.io/rfcs/002-wasm-bindgen-inheritance-casting.html
[extends-attr]: https://rustwasm.github.io/wasm-bindgen/reference/attributes/on-js-imports/extends.html
[#662]: https://github.com/rustwasm/wasm-bindgen/pull/662

--------------------------------------------------------------------------------

## 0.2.15

Released 2018-07-26.

### Fixed

* Fixed `wasm-bindgen` CLI version mismatch checks that got broken in the last
  point release.

--------------------------------------------------------------------------------

## 0.2.14

Released 2018-07-25.

### Fixed

* Fixed compilation errors on targets that use
  Mach-O. [#545](https://github.com/rustwasm/wasm-bindgen/issues/545)

--------------------------------------------------------------------------------

## 0.2.13

Released 2018-07-22.

### Added

* Support the `#[wasm_bindgen(js_name = foo)]` attribute on exported functions
  and methods to allow renaming an export to JS. This allows JS to call it by
  one name and Rust to call it by another, for example using `camelCase` in JS
  and `snake_case` in Rust

### Fixed

* Compilation with the latest nightly compiler has been fixed (nightlies on and
  after 2018-07-21)

--------------------------------------------------------------------------------

## 0.2.12

Released 2018-07-19.

This release is mostly internal refactorings and minor improvements to the
existing crates and functionality, but the bigs news is an upcoming `js-sys` and
`web-sys` set of crates. The `js-sys` crate will expose [all global JS
bindings][js-all] and the `web-sys` crate will be generated from WebIDL to
expose all APIs browsers have. More info on this soon!

[js-all]: https://github.com/rustwasm/wasm-bindgen/issues/275

### Added

* Support for `Option<T>` was added where `T` can be a number of slices or
  imported types.
* Comments in Rust are now preserved in generated JS bindings, as well as
  comments being generated to indicate the types of arguments/return values.
* The online documentation has been reorganized [into a book][book].
* The generated JS is now formatted better by default for readability.
* A `--keep-debug` flag has been added to the CLI to retain debug sections by
  default. This happens by default when `--debug` is passed.

[book]: https://rustwasm.github.io/wasm-bindgen/

### Fixed

* Compilation with the latest nightly compiler has been fixed (nightlies on and
  after 2018-07-19)
* Declarations of an imported function in multiple crates have been fixed to not
  conflict.
* Compilation with `#![deny(missing_docs)]` has been fixed.

--------------------------------------------------------------------------------

## 0.2.11

Released 2018-05-24.

--------------------------------------------------------------------------------

## 0.2.10

Released 2018-05-17.

--------------------------------------------------------------------------------

## 0.2.9

Released 2018-05-11.
