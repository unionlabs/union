# `skip`

When attached to a `pub` struct field this indicates that field will not be exposed to JavaScript,
and neither getter nor setter will be generated in ES6 class.

```rust
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct Foo {
    pub bar: u32,

    #[wasm_bindgen(skip)]
    pub baz: u32,
}

#[wasm_bindgen]
impl Foo {
    pub fn new() -> Self {
        Foo {
            bar: 1,
            baz: 2
        }
    }
}
```

Here the `bar` field will be both readable and writable from JS, but the
`baz` field will be `undefined` in JS.

```js
import('./pkg/').then(rust => {
    let foo = rust.Foo.new();
    
    // bar is accessible by getter
    console.log(foo.bar);
    // field marked with `skip` is undefined
    console.log(foo.baz);      

    // you can shadow it
    foo.baz = 45;       
    // so accessing by getter will return `45`
    // but it won't affect real value in rust memory
    console.log(foo.baz);
});
```
