# `method`

The `method` attribute allows you to describe methods of imported JavaScript
objects. It is applied on a function that has `this` as its first parameter,
which is a shared reference to an imported JavaScript type.

```rust
#[wasm_bindgen]
extern "C" {
    type Set;

    #[wasm_bindgen(method)]
    fn has(this: &Set, element: &JsValue) -> bool;
}
```

This generates a `has` method on `Set` in Rust, which invokes the
`Set.prototype.has` method in JavaScript.

```rust
let set: Set = ...;
let elem: JsValue = ...;
if set.has(&elem) {
    ...
}
```
