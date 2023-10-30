# `wasm-bindgen-macro`

Implementation of the `#[wasm_bindgen]` attribute. See the `wasm-bindgen`
documentation for more information about what this macro does.

## Testing

Testing of this macro is done through "ui tests" in the `ui-tests` subdirectory
of this crate. Each Rust file in this folder is compiled with the `wasm_bindgen`
crate, and the `*.stderr` file sitting next to it is the asserted output of the
compiler. If the output matches, the test passes, and if the output doesn't
match the test fails. Note that it is also considered a failure if a test
actually compiles successfully.

To add a test:

* Create `ui-tests/my-awesome-test.rs`
* Write an invalid `#[wasm_bindgen]` invocation, testing the error you're
  generating
* Execute `cargo test -p ui-tests`, the test will fail
* From within the `ui-tests` folder, execute `./update-all-references.sh`. This
  should create a `my-awesome-test.stderr` file.
* Inspect `my-awesome-test.stderr` to make sure it looks ok
* Rerun `cargo test -p ui-tests` and your tests should pass!

Testing here is a work in progress, see
[#601](https://github.com/rustwasm/wasm-bindgen/issues/601) for more
information.
