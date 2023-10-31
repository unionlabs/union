# WebAudio

[View full source code][code] or [view the compiled example online][online]

[online]: https://rustwasm.github.io/wasm-bindgen/exbuild/webaudio/
[code]: https://github.com/rustwasm/wasm-bindgen/tree/master/examples/webaudio

This example creates an [FM
oscillator](https://en.wikipedia.org/wiki/Frequency_modulation_synthesis) using
the [WebAudio
API](https://developer.mozilla.org/en-US/docs/Web/API/Web_Audio_API) and
`web-sys`.

## `Cargo.toml`

The `Cargo.toml` enables the types needed to use the relevant bits of the
WebAudio API.

```toml
{{#include ../../../examples/webaudio/Cargo.toml}}
```

## `src/lib.rs`

The Rust code implements the FM oscillator.

```rust
{{#include ../../../examples/webaudio/src/lib.rs}}
```

## `index.js`

A small bit of JavaScript glues the rust module to input widgets and translates
events into calls into wasm code.

```js
{{#include ../../../examples/webaudio/index.js}}
```
