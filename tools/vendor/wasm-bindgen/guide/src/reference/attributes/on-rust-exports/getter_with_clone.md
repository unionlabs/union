# `getter_with_clone`

By default, Rust exports exposed to JavaScript will generate getters that require fields to implement `Copy`. The `getter_with_clone` attribute can be used to generate getters that require `Clone` instead. This attribute can be applied per struct or per field. For example:

```rust
#[wasm_bindgen]
pub struct Foo {
    #[wasm_bindgen(getter_with_clone)]
    pub bar: String,
}

#[wasm_bindgen(getter_with_clone)]
pub struct Foo {
    pub bar: String,
    pub baz: String,
}
```
