# `static_method_of = Blah`

The `static_method_of` attribute allows one to specify that an imported function
is a static method of the given imported JavaScript class. For example, to bind
to JavaScript's `Date.now()` static method, one would use this attribute:

```rust
#[wasm_bindgen]
extern "C" {
    type Date;

    #[wasm_bindgen(static_method_of = Date)]
    pub fn now() -> f64;
}
```

The `now` function becomes a static method of the imported type in the Rust
bindings as well:

```rust
let instant = Date::now();
```

This is similar to the `js_namespace` attribute, but the usage from within Rust
is different since the method also becomes a static method of the imported type.
Additionally this attribute also specifies that the `this` parameter when
invoking the method is expected to be the JS class, e.g. always invoked as
`Date.now()` instead of `const x = Date.now; x()`.
