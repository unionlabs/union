# `readonly`

When attached to a `pub` struct field this indicates that it's read-only from
JavaScript, and a setter will not be generated and exported to JavaScript.

```rust
#[wasm_bindgen]
pub fn make_foo() -> Foo {
    Foo {
        first: 10,
        second: 20,
    }
}

#[wasm_bindgen]
pub struct Foo {
    pub first: u32,

    #[wasm_bindgen(readonly)]
    pub second: u32,
}
```

Here the `first` field will be both readable and writable from JS, but the
`second` field will be a `readonly` field in JS where the setter isn't
implemented and attempting to set it will throw an exception.

```js
import { make_foo } from "./my_module";

const foo = make_foo();

// Can both get and set `first`.
foo.first = 99;
console.log(foo.first);

// Can only get `second`.
console.log(foo.second);
```
