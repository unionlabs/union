# Using `wasm-bindgen-test`

### Add `wasm-bindgen-test` to Your `Cargo.toml`'s `[dev-dependencies]`

```toml
[dev-dependencies]
wasm-bindgen-test = "0.3.0"
```

Note that the `0.3.0` track of `wasm-bindgen-test` supports Rust 1.39.0+, which
is currently the nightly channel (as of 2019-09-05). If you want support for
older compilers use the `0.2.*` track of `wasm-bindgen-test`.

## Write Some Tests

Create a `$MY_CRATE/tests/wasm.rs` file:

```rust
use wasm_bindgen_test::*;

#[wasm_bindgen_test]
fn pass() {
    assert_eq!(1, 1);
}

#[wasm_bindgen_test]
fn fail() {
    assert_eq!(1, 2);
}
```

Writing tests is the same as normal Rust `#[test]`s, except we are using the
`#[wasm_bindgen_test]` attribute.

One other difference is that the tests **must** be in the root of the crate, or
within a `pub mod`. Putting them inside a private module will not work.

## Execute Your Tests

Run the tests with `wasm-pack test`. By default, the tests are generated to
target Node.js, but you can [configure tests to run inside headless
browsers](./browsers.html) as well.

```shell
$ wasm-pack test --node
    Finished dev [unoptimized + debuginfo] target(s) in 0.11s
     Running /home/.../target/wasm32-unknown-unknown/debug/deps/wasm-4a309ffe6ad80503.wasm
running 2 tests

test wasm::pass ... ok
test wasm::fail ... FAILED

failures:

---- wasm::fail output ----
    error output:
        panicked at 'assertion failed: `(left == right)`
          left: `1`,
         right: `2`', crates/test/tests/wasm.rs:14:5

    JS exception that was thrown:
        RuntimeError: unreachable
            at __rust_start_panic (wasm-function[1362]:33)
            at rust_panic (wasm-function[1357]:30)
            at std::panicking::rust_panic_with_hook::h56e5e464b0e7fc22 (wasm-function[1352]:444)
            at std::panicking::continue_panic_fmt::had70ba48785b9a8f (wasm-function[1350]:122)
            at std::panicking::begin_panic_fmt::h991e7d1ca9bf9c0c (wasm-function[1351]:95)
            at wasm::fail::ha4c23c69dfa0eea9 (wasm-function[88]:477)
            at core::ops::function::FnOnce::call_once::h633718dad359559a (wasm-function[21]:22)
            at wasm_bindgen_test::__rt::Context::execute::h2f669104986475eb (wasm-function[13]:291)
            at __wbg_test_fail_1 (wasm-function[87]:57)
            at module.exports.__wbg_apply_2ba774592c5223a7 (/home/alex/code/wasm-bindgen/target/wasm32-unknown-unknown/wbg-tmp/wasm-4a309ffe6ad80503.js:61:66)


failures:

    wasm::fail

test result: FAILED. 1 passed; 1 failed; 0 ignored

error: test failed, to rerun pass '--test wasm'
```

That's it!

--------------------------------------------------------------------------------

## Appendix: Using `wasm-bindgen-test` without `wasm-pack`

**⚠️ The recommended way to use `wasm-bindgen-test` is with `wasm-pack`, since it
will handle installing the test runner, installing a WebDriver client for your
browser, and informing `cargo` how to use the custom test runner.** However, you
can also manage those tasks yourself, if you wish.

In addition to the steps above, you must also do the following.

### Install the Test Runner

The test runner comes along with the main `wasm-bindgen` CLI tool. Make sure to
replace "X.Y.Z" with the same version of `wasm-bindgen` that you already have in
`Cargo.toml`!

```shell
cargo install wasm-bindgen-cli --vers "X.Y.Z"
```

### Configure `.cargo/config` to use the Test Runner

Add this to `$MY_CRATE/.cargo/config`:

```toml
[target.wasm32-unknown-unknown]
runner = 'wasm-bindgen-test-runner'
```

### Run the Tests

Run the tests by passing `--target wasm32-unknown-unknown` to `cargo test`:

```
cargo test --target wasm32-unknown-unknown
```
