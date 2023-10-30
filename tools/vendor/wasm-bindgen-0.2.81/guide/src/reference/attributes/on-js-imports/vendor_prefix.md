# Vendor-prefixed APIs

On the web new APIs often have vendor prefixes while they're in an experimental
state. For example the `AudioContext` API is known as `webkitAudioContext` in
Safari at the time of this writing. The `vendor_prefix` attribute indicates
these alternative names, which are used if the normal name isn't defined.

For example to use `AudioContext` you might do:

```rust
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(vendor_prefix = webkit)]
    type AudioContext;

    // methods on `AudioContext` ...
}
```

Whenever `AudioContext` is used it'll use `AudioContext` if the global namespace
defines it or alternatively it'll fall back to `webkitAudioContext`.

Note that `vendor_prefix` cannot be used with `module = "..."` or
`js_namespace = ...`, so it's basically limited to web-platform APIs today.
