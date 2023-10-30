# `catch`

The `catch` attribute allows catching a JavaScript exception. This can be
attached to any imported function or method, and the function must return a
`Result` where the `Err` payload is a `JsValue`:

```rust
#[wasm_bindgen]
extern "C" {
    // `catch` on a standalone function.
    #[wasm_bindgen(catch)]
    fn foo() -> Result<(), JsValue>;

    // `catch` on a method.
    type Zoidberg;
    #[wasm_bindgen(catch, method)]
    fn woop_woop_woop(this: &Zoidberg) -> Result<u32, JsValue>;
}
```

If calling the imported function throws an exception, then `Err` will be
returned with the exception that was raised. Otherwise, `Ok` is returned with
the result of the function.

> By default `wasm-bindgen` will take no action when wasm calls a JS function
> which ends up throwing an exception. The wasm spec right now doesn't support
> stack unwinding and as a result Rust code **will not execute destructors**.
> This can unfortunately cause memory leaks in Rust right now, but as soon as
> wasm implements catching exceptions we'll be sure to add support as well!
