# Optimizing for Size with `wasm-bindgen`

The Rust and WebAssembly Working Group's [Game of Life tutorial][gol] has an
excellent section on [shrinking wasm code size][size], but there's a few
`wasm-bindgen`-specific items to mention as well!

First and foremost, `wasm-bindgen` is designed to be lightweight and a "pay only
for what you use" mentality. If you suspect that `wasm-bindgen` is bloating your
program that is a bug and we'd like to know about it! Please feel free to [file
an issue][issue], even if it's a question!

### What to profile

With `wasm-bindgen` there's a few different files to be measuring the size of.
The first of which is the output of the compiler itself, typically at
`target/wasm32-unknown-unknown/release/foo.wasm`. **This file is not optimized
for size and you should not measure it.** The output of the compiler when
linking with `wasm-bindgen` is by design larger than it needs to be, the
`wasm-bindgen` CLI tool will automatically strip all unneeded functionality out
of the binary.

This leaves us with two primary generated files to measure the size of:

* **Generated wasm** - after running the `wasm-bindgen` CLI tool you'll get a
  file in `--out-dir` that looks like `foo_bg.wasm`. This file is the final
  fully-finished artifact from `wasm-bindgen`, and it reflects the size of the
  app you'll be publishing. All the optimizations [mentioned in the code size
  tutorial][size] will help reduce the size of this binary, so feel free to go
  crazy!

* **Generated JS** - the other file after running `wasm-bindgen` is a `foo.js`
  file which is what's actually imported by other JS code. This file is already
  generated to be as small as possible (not including unneeded functionality).
  The JS, however, is not uglified or minified, but rather still human readable
  and debuggable. It's expected that you'll run an uglifier or bundler of the JS
  output to minimize it further in your application. If you spot a way we could
  reduce the output JS size further (or make it more amenable to bundler
  minification), please let us know!

### Example

As an example, the `wasm-bindgen` repository [contains an example][example]
about generating small wasm binaries and shows off how to generate a small wasm
file for adding two numbers.

[gol]: https://rustwasm.github.io/book/game-of-life/introduction.html
[size]: https://rustwasm.github.io/book/game-of-life/code-size.html
[issue]: https://github.com/rustwasm/wasm-bindgen/issues/new
[example]: https://rustwasm.github.io/docs/wasm-bindgen/examples/add.html
