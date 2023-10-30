# `web-sys`

The `web-sys` crate provides raw bindings to all of the Web's APIs, and its
source lives at `wasm-bindgen/crates/web-sys`.

The `web-sys` crate is **entirely** mechanically generated inside `build.rs`
using `wasm-bindgen`'s WebIDL frontend and the WebIDL interface definitions for
Web APIs. This means that `web-sys` isn't always the most ergonomic crate to
use, but it's intended to provide verified and correct bindings to the web
platform, and then better interfaces can be iterated on crates.io!

Documentation for the published version of this crate is available on
[docs.rs][docsrs] but you can also check out the [master branch
documentation][masterdoc] for the crate.

[docsrs]: https://docs.rs/web-sys
[masterdoc]: https://rustwasm.github.io/wasm-bindgen/api/web_sys/
