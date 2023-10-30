# Support for Weak References

By default wasm-bindgen does not use the [TC39 weak references
proposal](https://github.com/tc39/proposal-weakrefs). This proposal just
advanced to stage 4 at the time of this writing, but it will still stake some
time for support to percolate into all the major browsers.

Without weak references your JS integration may be susceptible to memory leaks
in Rust, for example:

* You could forget to call `.free()` on a JS object, leaving the Rust memory
  allocated.
* Rust closures converted to JS values (the `Closure` type) may not be executed
  and cleaned up.
* Rust closures have `Closure::{into_js_value,forget}` methods which explicitly
  do not free the underlying memory.

These issues are all solved with the weak references proposal in JS. The
`--weak-refs` flag to the `wasm-bindgen` CLI will enable usage of
`FinalizationRegistry` to ensure that all memory is cleaned up, regardless of
whether it's explicitly deallocated or not. Note that explicit deallocation
is always a possibility and supported, but if it's not called then memory will
still be automatically deallocated with the `--weak-refs` flag.
