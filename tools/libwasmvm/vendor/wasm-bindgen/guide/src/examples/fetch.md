# The `fetch` API

[View full source code][code] or [view the compiled example online][online]

[online]: https://rustwasm.github.io/wasm-bindgen/exbuild/fetch/
[code]: https://github.com/rustwasm/wasm-bindgen/tree/master/examples/fetch

This example uses the `fetch` API to make an HTTP request to the GitHub API and
then parses the resulting JSON.

## `Cargo.toml`

The `Cargo.toml` enables a number of features related to the `fetch` API and
types used: `Headers`, `Request`, etc.

```toml
{{#include ../../../examples/fetch/Cargo.toml}}
```

## `src/lib.rs`

```rust
{{#include ../../../examples/fetch/src/lib.rs}}
```
