# Use of `static` to Access JS Objects

JavaScript modules will often export arbitrary static objects for use with
their provided interfaces. These objects can be accessed from Rust by declaring
a named `static` in the `extern` block. `wasm-bindgen` will bind a `JsStatic`
for these objects, which can be cloned into a `JsValue`. For example, given the
following JavaScript:

```js
let COLORS = {
    red: 'rgb(255, 0, 0)',
    green: 'rgb(0, 255, 0)',
    blue: 'rgb(0, 0, 255)',
};
```

`static` can aid in the access of this object from Rust:

```rust
#[wasm_bindgen]
extern "C" {
    static COLORS;
}

fn get_colors() -> JsValue {
    COLORS.clone()
}
```

Since `COLORS` is effectively a JavaScript namespace, we can use the same
mechanism to refer directly to namespaces exported from JavaScript modules, and
even to exported classes:

```js
let namespace = {
    // Members of namespace...
};

class SomeType {
    // Definition of SomeType...
};

export { SomeType, namespace };
```

The binding for this module:

```rust
#[wasm_bindgen(module = "/js/some-rollup.js")]
extern "C" {
    // Likewise with the namespace--this refers to the object directly.
    #[wasm_bindgen(js_name = namespace)]
    static NAMESPACE: JsValue;

    // Refer to SomeType's class
    #[wasm_bindgen(js_name = SomeType)]
    static SOME_TYPE: JsValue;

    // Other bindings for SomeType
    type SomeType;
    #[wasm_bindgen(constructor)]
    fn new() -> SomeType;
}
```
