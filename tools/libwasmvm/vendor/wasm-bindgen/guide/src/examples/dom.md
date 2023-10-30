# web-sys: DOM hello world

[View full source code][code] or [view the compiled example online][online]

[online]: https://rustwasm.github.io/wasm-bindgen/exbuild/dom/
[code]: https://github.com/rustwasm/wasm-bindgen/tree/master/examples/dom

Using `web-sys` we're able to interact with all the standard web platform
methods, including those of the DOM! Here we take a look at a simple "Hello,
world!" which manufactures a DOM element in Rust, customizes it, and then
appends it to the page.

## `Cargo.toml`

You can see here how we depend on `web-sys` and activate associated features to
enable all the various APIs:

```toml
{{#include ../../../examples/dom/Cargo.toml}}
```

## `src/lib.rs`

```rust
{{#include ../../../examples/dom/src/lib.rs}}
```
