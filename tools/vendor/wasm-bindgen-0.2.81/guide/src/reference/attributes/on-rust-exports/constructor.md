# `constructor`

When attached to a Rust "constructor" it will make the generated JavaScript
bindings callable as `new Foo()`.

For example, consider this exported Rust type and `constructor` annotation:

```rust
#[wasm_bindgen]
pub struct Foo {
    contents: u32,
}

#[wasm_bindgen]
impl Foo {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Foo {
        Foo { contents: 0 }
    }

    pub fn get_contents(&self) -> u32 {
        self.contents
    }
}
```

This can be used in JavaScript as:

```js
import { Foo } from './my_module';

const f = new Foo();
console.log(f.get_contents());
```
