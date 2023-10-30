# Hello, World!

[View full source code][code] or [view the compiled example online][online]

[online]: https://rustwasm.github.io/wasm-bindgen/exbuild/hello_world/
[code]: https://github.com/rustwasm/wasm-bindgen/tree/master/examples/hello_world

This is the "Hello, world!" example of `#[wasm_bindgen]` showing how to set up
a project, export a function to JS, call it from JS, and then call the `alert`
function in Rust.

## `Cargo.toml`

The `Cargo.toml` lists the `wasm-bindgen` crate as a dependency.

Also of note is the `crate-type = ["cdylib"]` which is largely used for wasm
final artifacts today.

```toml
{{#include ../../../examples/hello_world/Cargo.toml}}
```

## `src/lib.rs`

Here we define our Rust entry point along with calling the `alert` function.

```rust
{{#include ../../../examples/hello_world/src/lib.rs}}
```

## `index.js`

Our JS entry point is quite small!

```js
{{#include ../../../examples/hello_world/index.js}}
```

## Webpack-specific files

> **Note**: Webpack is required for this example, and if you're interested
> in options that don't use a JS bundler [see other examples][wab].

[wab]: without-a-bundler.html

And finally here's the Webpack configuration and `package.json` for this
project:

**webpack.config.js**

```js
{{#include ../../../examples/hello_world/webpack.config.js}}
```

**package.json**

```json
{{#include ../../../examples/hello_world/package.json}}
```
