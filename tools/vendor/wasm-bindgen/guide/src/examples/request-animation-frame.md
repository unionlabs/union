# `web-sys`: A `requestAnimationFrame` Loop

[View full source code][code] or [view the compiled example online][online]

[online]: https://rustwasm.github.io/wasm-bindgen/exbuild/request-animation-frame/
[code]: https://github.com/rustwasm/wasm-bindgen/tree/master/examples/request-animation-frame

This is an example of a `requestAnimationFrame` loop using the `web-sys` crate!
It renders a count of how many times a `requestAnimationFrame` callback has been
invoked and then it breaks out of the `requestAnimationFrame` loop after 300
iterations.

## `Cargo.toml`

You can see here how we depend on `web-sys` and activate associated features to
enable all the various APIs:

```toml
{{#include ../../../examples/request-animation-frame/Cargo.toml}}
```

## `src/lib.rs`

```rust
{{#include ../../../examples/request-animation-frame/src/lib.rs}}
```
