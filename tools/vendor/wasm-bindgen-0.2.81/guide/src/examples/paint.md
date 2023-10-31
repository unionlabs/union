# Paint Example

[View full source code][code] or [view the compiled example online][online]

[online]: https://rustwasm.github.io/wasm-bindgen/exbuild/paint/
[code]: https://github.com/rustwasm/wasm-bindgen/tree/master/examples/paint

A simple painting program.

## `Cargo.toml`

The `Cargo.toml` enables features necessary to work with the DOM, events and
2D canvas.

```toml
{{#include ../../../examples/paint/Cargo.toml}}
```

## `src/lib.rs`

Creates the `<canvas>` element, applies a CSS style to it, adds it to the document,
get a 2D rendering context and adds listeners for mouse events.

```rust
{{#include ../../../examples/paint/src/lib.rs}}
```
