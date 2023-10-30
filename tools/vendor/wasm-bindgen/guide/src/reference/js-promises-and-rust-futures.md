# Working with a JS `Promise` and a Rust `Future`

Many APIs on the web work with a `Promise`, such as an `async` function in JS.
Naturally you'll probably want to interoperate with them from Rust! To do that
you can use the `wasm-bindgen-futures` crate as well as Rust `async`
functions.

The first thing you might encounter is the need for working with a `Promise`.
For this you'll want to use [`js_sys::Promise`]. Once you've got one of those
values you can convert that value to `wasm_bindgen_futures::JsFuture`. This type
implements the `std::future::Future` trait which allows naturally using it in an
`async` function. For example:

[`js_sys::Promise`]: https://docs.rs/js-sys/*/js_sys/struct.Promise.html

```rust
async fn get_from_js() -> Result<JsValue, JsValue> {
    let promise = js_sys::Promise::resolve(&42.into());
    let result = wasm_bindgen_futures::JsFuture::from(promise).await?;
    Ok(result)
}
```

Here we can see how converting a `Promise` to Rust creates a `impl Future<Output
= Result<JsValue, JsValue>>`. This corresponds to `then` and `catch` in JS where
a successful promise becomes `Ok` and an erroneous promise becomes `Err`.

You can also import a JS async function directly with a `extern "C"` block, and
the promise will be converted to a future automatically. For now the return type
must be `JsValue` or no return at all:

```rust
#[wasm_bindgen]
extern "C" {
    async fn async_func_1() -> JsValue;
    async fn async_func_2();
}
```

The `async` can be combined with the `catch` attribute to manage errors from the
JS promise:

```rust
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(catch)]
    async fn async_func_3() -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch)]
    async fn async_func_4() -> Result<(), JsValue>;
}
```

Next up you'll probably want to export a Rust function to JS that returns a
promise. To do this you can use an `async` function and `#[wasm_bindgen]`:

```rust
#[wasm_bindgen]
pub async fn foo() {
    // ...
}
```

When invoked from JS the `foo` function here will return a `Promise`, so you can
import this as:

```js
import { foo } from "my-module";

async function shim() {
    const result = await foo();
    // ...
}
```

## Return values of `async fn`

When using an `async fn` in Rust and exporting it to JS there's some
restrictions on the return type. The return value of an exported Rust function
will eventually become `Result<JsValue, JsValue>` where `Ok` turns into a
successfully resolved promise and `Err` is equivalent to throwing an exception.

The following types are supported as return types from an `async fn`:

* `()` - turns into a successful `undefined` in JS
* `T: Into<JsValue>` - turns into a successful JS value
* `Result<(), E: Into<JsValue>>` - if `Ok(())` turns into a successful
  `undefined` and otherwise turns into a failed promise with `E` converted to a
  JS value
* `Result<T: Into<JsValue>, E: Into<JsValue>>` - like the previous case except
  both data payloads are converted into a `JsValue`.

Note that many types implement being converted into a `JsValue`, such as all
imported types via `#[wasm_bindgen]` (aka those in `js-sys` or `web-sys`),
primitives like `u32`, and all exported `#[wasm_bindgen]` types. In general,
you should be able to write code without having too many explicit conversions,
and the macro should take care of the rest!

## Using `wasm-bindgen-futures`

The `wasm-bindgen-futures` crate bridges the gap between JavaScript `Promise`s
and Rust `Future`s. Its `JsFuture` type provides conversion from a JavaScript
`Promise` into a Rust `Future`, and its `future_to_promise` function converts a
Rust `Future` into a JavaScript `Promise` and schedules it to be driven to
completion.

Learn more:

* [`wasm_bindgen_futures` on crates.io][crate]
* [`wasm-bindgen-futures` API documentation and example usage][docs]

[crate]: https://crates.io/crates/wasm-bindgen-futures
[docs]: https://rustwasm.github.io/wasm-bindgen/api/wasm_bindgen_futures/

## Compatibility with versions of `Future`

The current crate on crates.io, `wasm-bindgen-futures 0.4.*`, supports
`std::future::Future` and `async`/`await` in Rust. This typically requires Rust
1.39.0+ (as of this writing on 2019-09-05 it's the nightly channel of Rust).

If you're using the `Future` trait from the `futures` `0.1.*` crate then you'll
want to use the `0.3.*` track of `wasm-bindgen-futures` on crates.io.
