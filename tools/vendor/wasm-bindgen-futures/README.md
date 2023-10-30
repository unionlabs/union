# `wasm-bindgen-futures`

[API Documention][docs]

This crate bridges the gap between a Rust `Future` and a JavaScript
`Promise`. It provides two conversions:

1. From a JavaScript `Promise` into a Rust `Future`.
2. From a Rust `Future` into a JavaScript `Promise`.

Additionally under the feature flag `futures-core-03-stream` there is experimental 
support for `AsyncIterator` to `Stream` conversion.

See the [API documentation][docs] for more info.

[docs]: https://rustwasm.github.io/wasm-bindgen/api/wasm_bindgen_futures/
