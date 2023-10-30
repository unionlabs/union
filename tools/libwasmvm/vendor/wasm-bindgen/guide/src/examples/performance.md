# web-sys: `performance.now`

[View full source code][code] or [view the compiled example online][online]

[online]: https://rustwasm.github.io/wasm-bindgen/exbuild/performance/
[code]: https://github.com/rustwasm/wasm-bindgen/tree/master/examples/performance

Want to profile some Rust code in the browser? No problem! You can use the
`performance.now()` API and friends to get timing information to see how long
things take.

## `src/lib.rs`

```rust
{{#include ../../../examples/performance/src/lib.rs}}
```
