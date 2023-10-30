# Cargo Features in `web-sys`

To keep `web-sys` building as fast as possible, there is a cargo feature for
every type defined in `web-sys`. To access that type, you must enable its
feature. To access a method, you must enable the feature for its `self` type and
the features for each of its argument types. In the [API documentation][], every
method lists the features that are required to enable it.

For example, [the `WebGlRenderingContext::compile_shader` function][compile_shader] requires these features:

* `WebGlRenderingContext`, because that is the method's `self` type
* `WebGlShader`, because it takes an argument of that type

[API documentation]: https://rustwasm.github.io/wasm-bindgen/api/web_sys
[compile_shader]: https://rustwasm.github.io/wasm-bindgen/api/web_sys/struct.WebGlRenderingContext.html#method.compile_shader
