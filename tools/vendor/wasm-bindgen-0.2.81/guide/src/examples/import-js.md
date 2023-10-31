# Importing non-browser JS

[View full source code][code] or [view the compiled example online][online]

[online]: https://rustwasm.github.io/wasm-bindgen/exbuild/import_js/
[code]: https://github.com/rustwasm/wasm-bindgen/tree/master/examples/import_js

The `#[wasm_bindgen]` attribute can be used on `extern "C" { .. }` blocks to import
functionality from JS. This is how the `js-sys` and the `web-sys` crates are
built, but you can also use it in your own crate!

For example if you're working with this JS file:

```js
// defined-in-js.js
{{#include ../../../examples/import_js/crate/defined-in-js.js}}
```

you can use it in Rust with:

```rust
{{#include ../../../examples/import_js/crate/src/lib.rs}}
```

You can also [explore the full list of ways to configure imports][attr]

[attr]: ../reference/attributes/on-js-imports/index.html
