# Testing

You can test the `js-sys` crate by running `cargo test --target
wasm32-unknown-unknown` within the `crates/js-sys` directory in the
`wasm-bindgen` repository:

```sh
cd wasm-bindgen/crates/js-sys
cargo test --target wasm32-unknown-unknown
```

These tests are largely executed in Node.js right now via the
[`wasm-bindgen-test` framework](../../wasm-bindgen-test/index.html)
