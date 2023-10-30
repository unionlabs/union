# Testing

You can test the `web-sys` crate by running `cargo test` within the
`crates/web-sys` directory in the `wasm-bindgen` repository:

```sh
cd wasm-bindgen/crates/web-sys
cargo test --target wasm32-unknown-unknown --all-features
```

The Wasm tests all run within a headless browser. See [the `wasm-bindgen-test`
crate's
`README.md`](https://github.com/rustwasm/wasm-bindgen/blob/master/crates/test/README.md)
for details and configuring which headless browser is used.
