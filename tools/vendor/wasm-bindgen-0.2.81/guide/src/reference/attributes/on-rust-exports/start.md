# `start`

When attached to a `pub` function this attribute will configure the `start`
section of the wasm executable to be emitted, executing the tagged function as
soon as the wasm module is instantiated.

```rust
#[wasm_bindgen(start)]
pub fn main() {
    // executed automatically ...
}
```

The `start` section of the wasm executable will be configured to execute the
`main` function here as soon as it can. Note that due to various practical
limitations today the start section of the executable may not literally point to
`main`, but the `main` function here should be started up automatically when the
wasm module is loaded.

There's a few caveats to be aware of when using the `start` attribute:

* The `start` function must take no arguments and must either return `()` or
  `Result<(), JsValue>`
* Only one `start` function can be placed into a module, including its
  dependencies. If more than one is specified then `wasm-bindgen` will fail when
  the CLI is run. It's recommended that only applications use this attribute.
* The `start` function will not be executed when testing.
* If you're experimenting with WebAssembly threads, the `start` function is
  executed *once per thread*, not once globally!
* Note that the `start` function is relatively new, so if you find any bugs with
  it, please feel free to report an issue!
