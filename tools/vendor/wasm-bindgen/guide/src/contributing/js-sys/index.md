# `js-sys`

The [`js-sys` crate][js-sys] provides raw bindings to all the global APIs
guaranteed to exist in every JavaScript environment by the ECMAScript standard,
and its source lives at [`wasm-bindgen/crates/js-sys`][src].  With the `js-sys`
crate, we can work with `Object`s, `Array`s, `Function`s, `Map`s, `Set`s,
etc... without writing the `#[wasm_bindgen]` imports by hand.

Documentation for the published version of this crate is available on
[docs.rs][docsrs] but you can also check out the [master branch
documentation][masterdoc] for the crate.

[docsrs]: https://docs.rs/js-sys
[masterdoc]: https://rustwasm.github.io/wasm-bindgen/api/js_sys/
[src]: https://github.com/rustwasm/wasm-bindgen/tree/master/crates/js-sys

For example, we can invoke JavaScript [`Function`][mdn-function] callbacks and
time how long they take to execute with [`Date.now()`][mdn-date-now], and we
don't need to write any JS imports ourselves:

```rust
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn timed(callback: &js_sys::Function) -> f64 {
    let then = js_sys::Date::now();
    callback.apply(JsValue::null(), &js_sys::Array::new()).unwrap();
    let now = js_sys::Date::now();
    now - then
}
```

The `js-sys` crate doesn't contain bindings to any Web APIs like
[`document.querySelectorAll`][mdn-qsa]. These will be part of the
[`web-sys`][web-sys] crate.

[MDN]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects
[js-sys]: https://crates.io/crates/js-sys
[issue]: https://github.com/rustwasm/wasm-bindgen/issues/275
[mdn-function]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Function
[mdn-qsa]: https://developer.mozilla.org/en-US/docs/Web/API/Document/querySelectorAll
[web-sys]: https://crates.io/crates/web-sys
[web-sys-contributing]: https://rustwasm.github.io/wasm-bindgen/web-sys.html
[web-sys-issues]: https://github.com/rustwasm/wasm-bindgen/issues?q=is%3Aissue+is%3Aopen+label%3Aweb-sys
[mdn-date-now]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Date/now
