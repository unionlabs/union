# js-sys: WebAssembly in WebAssembly

[View full source code][code] or [view the compiled example online][online]

[online]: https://rustwasm.github.io/wasm-bindgen/exbuild/wasm-in-wasm/
[code]: https://github.com/rustwasm/wasm-bindgen/tree/master/examples/wasm-in-wasm

Using the `js-sys` crate we can get pretty meta and instantiate `WebAssembly`
modules from inside `WebAssembly` modules!

## `src/lib.rs`

```rust
{{#include ../../../examples/wasm-in-wasm/src/lib.rs}}
```
