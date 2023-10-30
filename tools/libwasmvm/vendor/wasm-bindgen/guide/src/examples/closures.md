# web-sys: Closures

[View full source code][code] or [view the compiled example online][online]

[online]: https://rustwasm.github.io/wasm-bindgen/exbuild/closures/
[code]: https://github.com/rustwasm/wasm-bindgen/tree/master/examples/closures

One of the features of `#[wasm_bindgen]` is that you can pass closures defined
in Rust off to JS. This can be a bit tricky at times, though, so the example
here shows how to interact with some standard web APIs with closures.

## `src/lib.rs`

```rust
{{#include ../../../examples/closures/src/lib.rs}}
```
