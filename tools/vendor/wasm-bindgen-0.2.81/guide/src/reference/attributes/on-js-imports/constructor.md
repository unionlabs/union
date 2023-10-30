# `constructor`

The `constructor` attribute is used to indicate that the function being bound
should actually translate to calling the `new` operator in JavaScript. The final
argument must be a type that's imported from JavaScript, and it's what will get
used in the generated glue:

```rust
#[wasm_bindgen]
extern "C" {
    type Shoes;

    #[wasm_bindgen(constructor)]
    fn new() -> Shoes;
}
```

This will attach a `new` static method to the `Shoes` type, and in JavaScript
when this method is called, it will be equivalent to `new Shoes()`.

```rust
// Become a cobbler; construct `new Shoes()`
let shoes = Shoes::new();
```
