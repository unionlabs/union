# `js-sys` Change Log

--------------------------------------------------------------------------------

## Unreleased

Released YYYY-MM-DD.

### Added

* TODO (or remove section if none)

### Changed

* TODO (or remove section if none)

### Deprecated

* TODO (or remove section if none)

### Removed

* TODO (or remove section if none)

### Fixed

* TODO (or remove section if none)

### Security

* TODO (or remove section if none)

--------------------------------------------------------------------------------

## 0.2.1

Released 2018-08-13.

### Added

* Added bindings to `Array.prototype.splice`.
* Added bindings to `RegExp`.
* Added bindings to `ArrayBuffer.prototype.byteLength`.
* Started adding the `#[wasm_bindgen(extends = ...)]` attribute to various JS
  types.
* Added bindings to `EvalError`.
* Added bindings to `Promise`. See the new `wasm-bindgen-futures` crate for
  integrating JS `Promise`s into Rust `Future`s.
* Added bindings to `JSON.{parse,stringify}`.
* Added bindings to `Array.of`.
* Added bindings to `Intl.Collator`.
* Added bindings to `Object.assign`.
* Added bindings to `Object.create`.
* Added bindings to `RangeError`.
* Added bindings to `ReferenceError`.
* Added bindings to `Symbol.unscopables`.
* Added bindings to `URIError`.
* Added bindings to `SyntaxError`.
* Added bindings to `TypeError`.

### Changed

* The `Intl` namespace was previously a bound object with static methods hanging
  off of it. It is now a module with free functions, and nested types.

--------------------------------------------------------------------------------

## 0.2.0

Released 2018-07-26.

Initial release!
