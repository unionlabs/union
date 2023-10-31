# Working with the `char` type

[View full source code][code] or [view the compiled example online][online]

[online]: https://rustwasm.github.io/wasm-bindgen/exbuild/char/
[code]: https://github.com/rustwasm/wasm-bindgen/tree/master/examples/char

The `#[wasm_bindgen]` macro will convert the rust `char` type to a single
code-point js `string`, and this example shows how to work with this.

Opening this example should display a single counter with a random character
for it's `key` and 0 for its `count`. You can click the `+` button to increase a
counter's count. By clicking on the "add counter" button you should see a new
counter added to the list with a different random character for it's `key`.

Under the hood javascript is choosing a random character from an Array of
characters and passing that to the rust Counter struct's constructor so the
character you are seeing on the page has made the full round trip from js to
rust and back to js.

## `src/lib.rs`

```rust
{{#include ../../../examples/char/src/lib.rs}}
```

## `index.js`

```js
{{#include ../../../examples/char/index.js}}
```
