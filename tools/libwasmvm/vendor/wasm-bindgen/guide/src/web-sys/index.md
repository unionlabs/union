# The `web-sys` Crate

[The `web-sys` crate][web-sys] provides raw `wasm-bindgen` imports for all of the Web's
APIs. This includes:

* `window.fetch`
* `Node.prototype.appendChild`
* WebGL
* WebAudio
* and many more!

It's sort of like the `libc` crate, but for the Web.

It does *not* include the JavaScript APIs that are guaranteed to exist in all
standards-compliant ECMAScript environments, such as `Array`, `Date`, and
`eval`. Bindings for these APIs can be found in [the `js-sys` crate][js-sys].

## API Documentation

[**Read the `web-sys` API documentation here!**][api]

[api]: https://rustwasm.github.io/wasm-bindgen/api/web_sys/
[js-sys]: https://crates.io/crates/js-sys
[web-sys]: https://crates.io/crates/web-sys
