# Examples of using `wasm-bindgen`, `js-sys`, and `web-sys`

This subsection contains examples of using the `wasm-bindgen`, `js-sys`, and
`web-sys` crates. Each example should have more information about what it's
doing.

These examples all assume familiarity with `wasm-bindgen`, `wasm-pack`, and
building a Rust and WebAssembly project. If you're unfamiliar with these check
out the [Game of Life tutorial][gol] or [wasm pack tutorials][wpt] to help you
get started.

The source code for all examples can also be [found online][code] to download
and run locally. Most examples are configured with Webpack/`wasm-pack` and can
be built with `npm run serve`. Other examples which don't use Webpack are
accompanied with instructions or a `build.sh` showing how to build it.

Note that most examples currently use Webpack to assemble the final output
artifact, but this is not required! You can review the [deployment
documentation][deploy] for other options of how to deploy Rust and WebAssembly.

[code]: https://github.com/rustwasm/wasm-bindgen/tree/master/examples
[gol]: https://rustwasm.github.io/docs/book/
[deploy]: ../reference/deployment.html
[wpt]: https://rustwasm.github.io/docs/wasm-pack/tutorials/index.html
