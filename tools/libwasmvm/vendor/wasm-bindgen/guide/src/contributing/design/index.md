# Design of `wasm-bindgen`

This section is intended to be a deep-dive into how `wasm-bindgen` internally
works today, specifically for Rust. If you're reading this far in the future it
may no longer be up to date, but feel free to open an issue and we can try to
answer questions and/or update this!

## Foundation: ES Modules

The first thing to know about `wasm-bindgen` is that it's fundamentally built on
the idea of ES Modules. In other words this tool takes an opinionated stance
that wasm files *should be viewed as ES modules*. This means that you can
`import` from a wasm file, use its `export`-ed functionality, etc, from normal
JS files.

Now unfortunately at the time of this writing the interface of wasm interop
isn't very rich. Wasm modules can only call functions or export functions that
deal exclusively with `i32`, `i64`, `f32`, and `f64`. Bummer!

That's where this project comes in. The goal of `wasm-bindgen` is to enhance the
"ABI" of wasm modules with richer types like classes, JS objects, Rust structs,
strings, etc. Keep in mind, though, that everything is based on ES Modules! This
means that the compiler is actually producing a "broken" wasm file of sorts. The
wasm file emitted by rustc, for example, does not have the interface we would
like to have. Instead it requires the `wasm-bindgen` tool to postprocess the
file, generating a `foo.js` and `foo_bg.wasm` file. The `foo.js` file is the
desired interface expressed in JS (classes, types, strings, etc) and the
`foo_bg.wasm` module is simply used as an implementation detail (it was
lightly modified from the original `foo.wasm` file).

As more features are stabilized in WebAssembly over time (like host bindings)
the JS file is expected to get smaller and smaller. It's unlikely to ever
disappear, but `wasm-bindgen` is designed to follow the WebAssembly spec and
proposals closely to optimize JS/Rust as much as possible.

## Foundation #2: Unintrusive in Rust

On the more Rust-y side of things the `wasm-bindgen` crate is designed to
ideally have as minimal impact on a Rust crate as possible. Ideally a few
`#[wasm_bindgen]` attributes are annotated in key locations and otherwise you're
off to the races. The attribute strives to both not invent new syntax and work
with existing idioms today.

For example a library might exposed a function in normal Rust that looks like:

```rust
pub fn greet(name: &str) -> String {
    // ...
}
```

And with `#[wasm_bindgen]` all you need to do in exporting it to JS is:

```rust
#[wasm_bindgen]
pub fn greet(name: &str) -> String {
    // ...
}
```

Additionally the design here with minimal intervention in Rust should allow us
to easily take advantage of the upcoming [host bindings][host] proposal. Ideally
you'd simply upgrade `wasm-bindgen`-the-crate as well as your toolchain and
you're immediately getting raw access to host bindings! (this is still a bit of
a ways off though...)

[host]: https://github.com/WebAssembly/host-bindings
