# Converting WebAssembly to JS

[View full source code][code]

[code]: https://github.com/rustwasm/wasm-bindgen/tree/master/examples/wasm2js

Not all browsers have support for `WebAssembly` at this time (although all major
ones do). If you'd like to support older browsers, you probably want a method
that doesn't involve keeping two codebases in sync!

Thankfully there's a tool from [binaryen] called `wasm2js` to convert a wasm
file to JS. This JS file, if successfully produced, is equivalent to the wasm
file (albeit a little bit larger and slower), and can be loaded into practically
any browser.

This example is relatively simple (cribbing from the [`console.log`
example](console-log.md)):

```rust
{{#include ../../../examples/wasm2js/src/lib.rs}}
```

The real magic happens when you actually build the app. Just after
`wasm-bindgen` we see here how we execute `wasm2js` in our build script:

```sh
{{#include ../../../examples/wasm2js/build.sh}}
```

Note that the `wasm2js` tool is still pretty early days so there's likely to be
a number of bugs to run into or work around. If any are encountered though
please feel free to report them upstream!

Also note that eventually this will ideally be automatically done by your
bundler and no action would be needed from you to work in older browsers via
`wasm2js`!

[binaryen]: https://github.com/WebAssembly/binaryen
