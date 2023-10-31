# Introduction

This book is about `wasm-bindgen`, a Rust library and CLI tool that facilitate
high-level interactions between wasm modules and JavaScript. The `wasm-bindgen`
tool and crate are only one part of the [Rust and WebAssembly
ecosystem][rustwasm]. If you're not familiar already with `wasm-bindgen` it's
recommended to start by reading the [Game of Life tutorial][gol]. If you're
curious about `wasm-pack`, you can find that [documentation here][wasm-pack].

The `wasm-bindgen` tool is sort of half polyfill for features like the [host
bindings proposal][host] and half features for empowering high-level
interactions between JS and wasm-compiled code (currently mostly from Rust).
More specifically this project allows JS/wasm to communicate with strings, JS
objects, classes, etc, as opposed to purely integers and floats. Using
`wasm-bindgen` for example you can define a JS class in Rust or take a string
from JS or return one. The functionality is growing as well!

Currently this tool is Rust-focused but the underlying foundation is
language-independent, and it's hoping that over time as this tool stabilizes
that it can be used for languages like C/C++!

Notable features of this project includes:

* Importing JS functionality in to Rust such as [DOM manipulation][dom-ex],
  [console logging][console-log], or [performance monitoring][perf-ex].
* Exporting Rust functionality to JS such as classes, functions, etc.
* Working with rich types like strings, numbers, classes, closures, and objects
  rather than simply `u32` and floats.
* Automatically generating TypeScript bindings for Rust code being consumed by
  JS.

With the addition of [`wasm-pack`][wasm-pack] you can run the gamut from running Rust on
the web locally, publishing it as part of a larger application, or even
publishing Rust-compiled-to-WebAssembly on NPM!

[host]: https://github.com/WebAssembly/host-bindings
[dom-ex]: https://github.com/rustwasm/wasm-bindgen/tree/master/examples/dom
[console-log]: https://github.com/rustwasm/wasm-bindgen/tree/master/examples/console_log
[perf-ex]: https://github.com/rustwasm/wasm-bindgen/tree/master/examples/performance
[hello-online]: https://webassembly.studio/?f=gzubao6tg3
[rustwasm]: https://rustwasm.github.io/
[gol]: https://rustwasm.github.io/docs/book/
[wasm-pack]: https://rustwasm.github.io/docs/wasm-pack/
