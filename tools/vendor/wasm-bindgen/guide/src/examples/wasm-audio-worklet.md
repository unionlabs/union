# WASM audio worklet

[View full source code][code] or [view the compiled example online][online]

[online]: https://wasm-bindgen.netlify.app/exbuild/wasm-audio-worklet/
[code]: https://github.com/rustwasm/wasm-bindgen/tree/master/examples/wasm-audio-worklet

This is an example of using threads inside specific worklets with WebAssembly,
Rust, and `wasm-bindgen`, culminating in an oscillator demo. This demo should
complement the [parallel-raytrace][parallel-raytrace] example by
demonstrating an alternative approach using ES modules with on-the-fly module
creation.

[parallel-raytrace]: https://rustwasm.github.io/docs/wasm-bindgen/examples/raytrace.html

### Building the demo

One of the major gotchas with threaded WebAssembly is that Rust does not ship a
precompiled target (e.g. standard library) which has threading support enabled.
This means that you'll need to recompile the standard library with the
appropriate rustc flags, namely
`-C target-feature=+atomics,+bulk-memory,+mutable-globals`.
Note that this requires a nightly Rust toolchain. See the [more detailed
instructions][build] of the parallel-raytrace example.

[build]: https://rustwasm.github.io/docs/wasm-bindgen/examples/raytrace.html#building-the-demo

### Caveats

This example shares most of its [caveats][caveats] with the parallel-raytrace
example. However, it tries to encapsulate worklet creation in a Rust module, so
the application developer does not need to maintain custom JS code.

[caveats]: https://rustwasm.github.io/docs/wasm-bindgen/examples/raytrace.html#caveats

### Browser Requirements

This demo should work in the latest Chrome and Safari versions at this time.
Firefox [does not support][firefox-worklet-import] imports in worklet modules,
which are difficult to avoid in this example, as `importScripts` is unavailable
in worklets. Note that this example requires HTTP headers to be set like in
[parallel-raytrace][headers].

[firefox-worklet-import]: https://bugzilla.mozilla.org/show_bug.cgi?id=1572644
[headers]: https://rustwasm.github.io/docs/wasm-bindgen/examples/raytrace.html#browser-requirements
