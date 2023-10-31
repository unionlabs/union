# `console.log`

[View full source code][code] or [view the compiled example online][online]

[online]: https://rustwasm.github.io/wasm-bindgen/exbuild/console_log/
[code]: https://github.com/rustwasm/wasm-bindgen/tree/master/examples/console_log

This example shows off how to use `console.log` in a variety of ways, all the
way from bare-bones usage to a `println!`-like macro with `web_sys`.

## `src/lib.rs`

```rust
{{#include ../../../examples/console_log/src/lib.rs}}
```
