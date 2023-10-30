# Function Overloads

Many Web APIs are overloaded to take different types of arguments or to skip
arguments completely. `web-sys` contains multiple bindings for these functions
that each specialize to a particular overload and set of argument types.

For example, [the `fetch` API][mdn-fetch] can be given a URL string, or a
`Request` object, and it might also optionally be given a `RequestInit` options
object. Therefore, we end up with these `web-sys` functions that all bind to the
`window.fetch` function:

* [`Window::fetch_with_str`](https://rustwasm.github.io/wasm-bindgen/api/web_sys/struct.Window.html#method.fetch_with_str)
* [`Window::fetch_with_request`](https://rustwasm.github.io/wasm-bindgen/api/web_sys/struct.Window.html#method.fetch_with_request)
* [`Window::fetch_with_str_and_init`](https://rustwasm.github.io/wasm-bindgen/api/web_sys/struct.Window.html#method.fetch_with_str_and_init)
* [`Window::fetch_with_request_and_init`](https://rustwasm.github.io/wasm-bindgen/api/web_sys/struct.Window.html#method.fetch_with_request_and_init)

Note that different overloads can use different interfaces, and therefore can
require different sets of cargo features to be enabled.

[mdn-fetch]: https://developer.mozilla.org/en-US/docs/Web/API/WindowOrWorkerGlobalScope/fetch
