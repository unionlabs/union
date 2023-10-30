# `inspectable`

By default, structs exported from Rust become JavaScript classes with a single `ptr` property. All other properties are implemented as getters, which are not displayed when calling `toJSON`.

The `inspectable` attribute can be used on Rust structs to provide a `toJSON` and `toString` implementation that display all readable fields. For example:

```rust
#[wasm_bindgen(inspectable)]
pub struct Baz {
    pub field: i32,
    private: i32,
}

#[wasm_bindgen]
impl Baz {
    #[wasm_bindgen(constructor)]
    pub fn new(field: i32) -> Baz {
        Baz { field, private: 13 }
    }
}
```

Provides the following behavior as in this JavaScript snippet:

```js
const obj = new Baz(3);
assert.deepStrictEqual(obj.toJSON(), { field: 3 });
obj.field = 4;
assert.strictEqual(obj.toString(), '{"field":4}');
```

One or both of these implementations can be overridden as desired. Note that the generated `toString` calls `toJSON` internally, so overriding `toJSON` will affect its output as a side effect.

```rust
#[wasm_bindgen]
impl Baz {
    #[wasm_bindgen(js_name = toJSON)]
    pub fn to_json(&self) -> i32 {
        self.field
    }

    #[wasm_bindgen(js_name = toString)]
    pub fn to_string(&self) -> String {
        format!("Baz: {}", self.field)
    }
}
```

Note that the output of `console.log` will remain unchanged and display only the `ptr` field in browsers. It is recommended to call `toJSON` or `JSON.stringify` in these situations to aid with logging or debugging. Node.js does not suffer from this limitation, see the section below.

## `inspectable` Classes in Node.js

When the `nodejs` target is used, an additional `[util.inspect.custom]` implementation is provided which calls `toJSON` internally. This method is used for `console.log` and similar functions to display all readable fields of the Rust struct.
