# `js_class = "Blah"`

The `js_class` attribute can be used in conjunction with the `method` attribute
to bind methods of imported JavaScript classes that have been renamed on the
Rust side.

```rust
#[wasm_bindgen]
extern "C" {
    // We don't want to import JS strings as `String`, since Rust already has a
    // `String` type in its prelude, so rename it as `JsString`.
    #[wasm_bindgen(js_name = String)]
    type JsString;

    // This is a method on the JavaScript "String" class, so specify that with
    // the `js_class` attribute.
    #[wasm_bindgen(method, js_class = "String", js_name = charAt)]
    fn char_at(this: &JsString, index: u32) -> JsString;
}
```
