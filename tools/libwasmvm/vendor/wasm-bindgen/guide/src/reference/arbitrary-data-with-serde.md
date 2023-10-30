# Serializing and Deserializing Arbitrary Data Into and From `JsValue` with Serde

It's possible to pass arbitrary data from Rust to JavaScript by serializing it
with [Serde](https://github.com/serde-rs/serde). This can be done through the
[`serde-wasm-bindgen`](https://docs.rs/serde-wasm-bindgen) crate.

## Add dependencies

To use `serde-wasm-bindgen`, you first have to add it as a dependency in your
`Cargo.toml`. You also need the `serde` crate, with the `derive` feature
enabled, to allow your types to be serialized and deserialized with Serde.

```toml
[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde-wasm-bindgen = "0.4"
```

## Derive the `Serialize` and `Deserialize` Traits

Add `#[derive(Serialize, Deserialize)]` to your type. All of your type's
members must also be supported by Serde, i.e. their types must also implement
the `Serialize` and `Deserialize` traits.

For example, let's say we'd like to pass this `struct` to JavaScript; doing so
is not possible in `wasm-bindgen` normally due to the use of `HashMap`s, arrays,
and nested `Vec`s. None of those types are supported for sending across the wasm
ABI naively, but all of them implement Serde's `Serialize` and `Deserialize`.

Note that we do not need to use the `#[wasm_bindgen]` macro.

```rust
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct Example {
    pub field1: HashMap<u32, String>,
    pub field2: Vec<Vec<f32>>,
    pub field3: [f32; 4],
}
```

## Send it to JavaScript with `serde_wasm_bindgen::to_value`

Here's a function that will pass an `Example` to JavaScript by serializing it to
`JsValue`:

```rust
#[wasm_bindgen]
pub fn send_example_to_js() -> JsValue {
    let mut field1 = HashMap::new();
    field1.insert(0, String::from("ex"));
    let example = Example {
        field1,
        field2: vec![vec![1., 2.], vec![3., 4.]],
        field3: [1., 2., 3., 4.]
    };

    serde_wasm_bindgen::to_value(&example).unwrap()
}
```

## Receive it from JavaScript with `serde_wasm_bindgen::from_value`

Here's a function that will receive a `JsValue` parameter from JavaScript and
then deserialize an `Example` from it:

```rust
#[wasm_bindgen]
pub fn receive_example_from_js(val: JsValue) {
    let example: Example = serde_wasm_bindgen::from_value(val).unwrap();
    ...
}
```

## JavaScript Usage

In the `JsValue` that JavaScript gets, `field1` will be a `Map`, `field2` will
be a JavaScript `Array` whose members are `Array`s of numbers, and `field3`
will be an `Array` of numbers.

```js
import { send_example_to_js, receive_example_from_js } from "example";

// Get the example object from wasm.
let example = send_example_to_js();

// Add another "Vec" element to the end of the "Vec<Vec<f32>>"
example.field2.push([5, 6]);

// Send the example object back to wasm.
receive_example_from_js(example);
```

## An alternative approach - using JSON

`serde-wasm-bindgen` works by directly manipulating JavaScript values. This
requires a lot of calls back and forth between Rust and JavaScript, which can
sometimes be slow. An alternative way of doing this is to serialize values to
JSON, and then parse them on the other end. Browsers' JSON implementations are
usually quite fast, and so this approach can outstrip `serde-wasm-bindgen`'s
performance in some cases. But this approach supports only types that can be
serialized as JSON, leaving out some important types that `serde-wasm-bindgen`
supports such as `Map`, `Set`, and array buffers.

That's not to say that using JSON is always faster, though - the JSON approach
can be anywhere from 2x to 0.2x the speed of `serde-wasm-bindgen`, depending on
the JS runtime and the values being passed. It also leads to larger code size
than `serde-wasm-bindgen`. So, make sure to profile each for your own use
cases.

This approach is implemented in [`gloo_utils::format::JsValueSerdeExt`]:

```toml
# Cargo.toml
[dependencies]
gloo-utils = { version = "0.1", features = ["serde"] }
```

```rust
use gloo_utils::format::JsValueSerdeExt;

#[wasm_bindgen]
pub fn send_example_to_js() -> JsValue {
    let mut field1 = HashMap::new();
    field1.insert(0, String::from("ex"));
    let example = Example {
        field1,
        field2: vec![vec![1., 2.], vec![3., 4.]],
        field3: [1., 2., 3., 4.]
    };

    JsValue::from_serde(&example).unwrap()
}

#[wasm_bindgen]
pub fn receive_example_from_js(val: JsValue) {
    let example: Example = val.into_serde().unwrap();
    ...
}
```

[`gloo_utils::format::JsValueSerdeExt`]: https://docs.rs/gloo-utils/latest/gloo_utils/format/trait.JsValueSerdeExt.html

## History

In previous versions of `wasm-bindgen`, `gloo-utils`'s JSON-based Serde support
(`JsValue::from_serde` and `JsValue::into_serde`) was built into `wasm-bindgen`
itself. However, this required a dependency on `serde_json`, which had a
problem: with certain features of `serde_json` and other crates enabled,
`serde_json` would end up with a circular dependency on `wasm-bindgen`, which
is illegal in Rust and caused people's code to fail to compile. So, these
methods were extracted out into `gloo-utils` with an extension trait and the
originals were deprecated.
