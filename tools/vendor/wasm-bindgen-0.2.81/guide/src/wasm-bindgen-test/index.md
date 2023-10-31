# Testing on `wasm32-unknown-unknown` with `wasm-bindgen-test`

The `wasm-bindgen-test` crate is an experimental test harness for Rust programs
compiled to wasm using `wasm-bindgen` and the `wasm32-unknown-unknown`
target.

## Goals

* Write tests for wasm as similar as possible to how you normally would write
  `#[test]`-style unit tests for native targets.

* Run the tests with the usual `cargo test` command but with an explicit wasm
  target:

  ```
  cargo test --target wasm32-unknown-unknown
  ```
