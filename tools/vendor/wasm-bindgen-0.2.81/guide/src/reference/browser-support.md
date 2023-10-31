# Supported Browsers

The output of `wasm-bindgen` includes a JS file, and as a result it's good to
know what browsers that file is expected to be used in! By default the output
uses ES modules which isn't implemented in all browsers today, but when using a
bundler (like Webpack) you should be able to produce output suitable for all
browsers.

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
  
* **Edge before 79+** - the `TextEncoder` and `TextDecoder` APIs, which
  `wasm-bindgen` uses to encode/decode strings between JS and Rust, were not
  available before version 79. You can polyfill this with at least one of two
  strategies:

  1. If using a bundler, you can likely configure the bundler to polyfill these
     types by default. For example if you're using Webpack you can use the
     [`ProvidePlugin` interface][wpp] like so after also adding
     [`text-encoding`] to your `package.json`

     ```js
     const webpack = require('webpack');
     module.exports = {
         plugins: [
             new webpack.ProvidePlugin({
               TextDecoder: ['text-encoding', 'TextDecoder'],
               TextEncoder: ['text-encoding', 'TextEncoder']
             })
         ]
         // ... other configuration options
     };
     ```

     **Warning:** doing this implies the polyfill will always be used,
     even if native APIs are available. This has a very significant
     performance impact (the polyfill was measured to be 100x slower in Chromium)!

  2. If you're not using a bundler you can also include support manually by
     adding a `<script>` tag which defines the `TextEncoder` and `TextDecoder`
     globals. [This StackOverflow question][soq] has some example usage and MDN
     has a [`TextEncoder` polyfill implementation][mdntepi] to get you started
     as well.

* **BigInt and `u64`** - currently the WebAssembly specification for the web
  forbids the usage of 64-bit integers (Rust types `i64` and `u64`) in
  exported/imported functions. When using `wasm-bindgen`, however, `u64` is
  allowed! The reason for this is that it's translated to the `BigInt` type in
  JS. The `BigInt` class is supported by all major browsers starting in the 
  following versions: Chrome 67+, Firefox 68+, Edge 79+, and Safari 14+.


If you find other incompatibilities please report them to us! We'd love to
either keep this list up-to-date or fix the underlying bugs :)

[report it]: https://github.com/rustwasm/wasm-bindgen/issues/new
[w2js]: https://github.com/WebAssembly/binaryen
[wpp]: https://webpack.js.org/plugins/provide-plugin/
[`text-encoding`]: https://www.npmjs.com/package/text-encoding
[soq]: https://stackoverflow.com/questions/40662142/polyfill-for-textdecoder/46549188#46549188
[mdntepi]: https://developer.mozilla.org/en-US/docs/Web/API/TextEncoder#Polyfill
