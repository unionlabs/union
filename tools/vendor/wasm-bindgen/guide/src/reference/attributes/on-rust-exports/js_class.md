# `js_class = Blah`

The `js_class` attribute is used to indicate that all the methods inside an
`impl` block should be attached to the specified JS class instead of inferring
it from the self type in the `impl` block. The `js_class` attribute is most
frequently paired with [the `js_name` attribute](js_name.html) on structs:

```rust
#[wasm_bindgen(js_name = Foo)]
pub struct JsFoo { /* ... */ }

#[wasm_bindgen(js_class = Foo)]
impl JsFoo {
    #[wasm_bindgen(constructor)]
    pub fn new() -> JsFoo { /* ... */ }

    pub fn foo(&self) { /* ... */ }
}
```

which is accessed like:

```rust
import { Foo } from './my_module';

const x = new Foo();
x.foo();
```
