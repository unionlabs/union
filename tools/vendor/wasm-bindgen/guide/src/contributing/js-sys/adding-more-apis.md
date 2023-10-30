# Adding Support for More JavaScript Global APIs

As of 2018-09-24 we've [added all APIs][issue] in the current ECMAScript
standard (yay!). To that end you'll hopefully not find a missing API, but if you
do please feel free to file an issue!

We currently add new APIs added to ECMAScript that are in [TC39 stage 4][tc39]
to this crate. If there's a new API in stage 4, feel free to file an issue as
well!

### Instructions for adding an API

* [ ] Find the `wasm-bindgen` issue for the API you'd like to add. If this
  doesn't exist, feel free to open one! Afterwards be sure to comment on the
  issue to avoid duplication of work.

* [ ] Open the [MDN
  page](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects)
  for the relevant JS API.

* [ ] Open `crates/js-sys/src/lib.rs` in your editor; this is the file where we
  are implementing the bindings.

* [ ] Follow the instructions in the top of `crates/js-sys/src/lib.rs` about how
  to add new bindings.

* [ ] Add a test for the new binding to `crates/js-sys/tests/wasm/MyType.rs`

* [ ] Run the [JS global API bindings tests][test]

* [ ] Send a pull request!

[issue]: https://github.com/rustwasm/wasm-bindgen/issues/275
[tc39]: https://tc39.github.io/process-document/
[test]: testing.html
