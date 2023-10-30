# Running `wasm-bindgen`'s Tests

## Wasm Tests on Node and Headless Browsers

These are the largest test suites, and most common to run in day to day
`wasm-bindgen` development. These tests are compiled to Wasm and then run in
Node.js or a headless browser via the WebDriver protocol.

```bash
cargo test --target wasm32-unknown-unknown
```

See [the `wasm-bindgen-test` crate's
`README.md`](https://github.com/rustwasm/wasm-bindgen/blob/master/crates/test/README.md)
for details and configuring which headless browser is used.

## Sanity Tests for `wasm-bindgen` on the Native Host Target

This small test suite just verifies that exported `wasm-bindgen` methods can
still be used on the native host's target.

```
cargo test
```

## The Web IDL Frontend's Tests

```
cargo test -p webidl-tests --target wasm32-unknown-unknown
```

## The Macro UI Tests

These tests assert that we have reasonable error messages that point to the
right source spans when the `#[wasm_bindgen]` proc-macro is misused.

```
cargo test -p ui-tests
```

## The `js-sys` Tests

See [the `js-sys` testing page](js-sys/testing.html).

## The `web-sys` Tests

See [the `web-sys` testing page](web-sys/testing.html).
