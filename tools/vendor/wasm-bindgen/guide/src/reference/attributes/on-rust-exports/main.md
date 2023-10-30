# `main`

When attached to the `main` function this attribute will adjust it to properly
throw errors if they can be.

```rust
#[wasm_bindgen(main)]
fn main() -> Result<(), JsValue> {
    Err(JsValue::from("this error message will be thrown"))
}
```

The attribute also allows using `async fn main()` in Cargo binaries.

```rust
#[wasm_bindgen(main)]
async fn main() {
    // ...
    future.await;
}
```

This attribute is only intended to be used on the `main` function of binaries or
examples. Unlike `#[wasm_bindgen(start)]`, it will not cause an arbitrary
function to be executed on start in a library.

The return type support is modeled after [`Termination`]. `()` and `Infallible`
are supported, but [`Termination`] itself is not. In order, wasm-bindgen will
first detect a `Result<(), impl Into<JsValue>>` and will throw proper
`JsValue`s, `Result<(), impl Debug>` will convert an error to a string and throw
that.

[`Termination`]: https://doc.rust-lang.org/std/process/trait.Termination.html
