# Supported Browsers

The output of `wasm-bindgen` includes a JS file, and as a result it's good to
know what browsers that file is expected to be used in! By default the output
uses ES modules with Wasm imports which isn't implemented in browsers today,
but when using a bundler (like Webpack) or `--target web` you should be able
to produce output suitable for all browsers.

Firefox, Chrome, Safari, and Edge browsers are all supported by
`wasm-bindgen`. If you find a problem in one of these browsers please [report
it] as we'd like to fix the bug! If you find a bug in another browser we would
also like to be aware of it!

## Caveats

* **IE 11** - `wasm-bindgen` by default requires support for
  `WebAssembly`, but no version of IE currently supports `WebAssembly`. You can
  support IE by [compiling wasm files to JS using `wasm2js`][w2js] (you can [see
  an example of doing this too](../examples/wasm2js.html)). Note
  that at this time no bundler will do this by default, but we'd love to
  document plugins which do this if you are aware of one!

If you find other incompatibilities please report them to us! We'd love to
either keep this list up-to-date or fix the underlying bugs :)

[report it]: https://github.com/rustwasm/wasm-bindgen/issues/new
[w2js]: https://github.com/WebAssembly/binaryen
