# Julia Set

[View full source code][code] or [view the compiled example online][online]

[online]: https://rustwasm.github.io/wasm-bindgen/exbuild/julia_set/
[code]: https://github.com/rustwasm/wasm-bindgen/tree/master/examples/julia_set

While not showing off a lot of `web_sys` API surface area, this example shows a
neat fractal that you can make!

## `index.js`

A small bit of glue is added for this example

```js
{{#include ../../../examples/julia_set/index.js}}
```

## `src/lib.rs`

The bulk of the logic is in the generation of the fractal

```rust
{{#include ../../../examples/julia_set/src/lib.rs}}
```
