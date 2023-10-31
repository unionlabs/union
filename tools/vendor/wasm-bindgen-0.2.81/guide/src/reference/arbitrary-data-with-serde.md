# Serializing and Deserializing Arbitrary Data Into and From `JsValue` with Serde

It's possible to pass arbitrary data from Rust to JavaScript by serializing it
to JSON with [Serde](https://github.com/serde-rs/serde). `wasm-bindgen` includes
the `JsValue` type, which streamlines serializing and deserializing.

## Enable the `"serde-serialize"` Feature

To enable the `"serde-serialize"` feature, do two things in `Cargo.toml`:

1. Add the `serde` and `serde_derive` crates to `[dependencies]`.
2. Add `features = ["serde-serialize"]` to the existing `wasm-bindgen`
   dependency.

```toml
[dependencies]
serde = { version = "1.0", features = ["derive"] }
wasm-bindgen = { version = "0.2", features = ["serde-serialize"] }
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

## Send it to JavaScript with `JsValue::from_serde`

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

    JsValue::from_serde(&example).unwrap()
}
```

## Receive it from JavaScript with `JsValue::into_serde`

Here's a function that will receive a `JsValue` parameter from JavaScript and
then deserialize an `Example` from it:

```rust
#[wasm_bindgen]
pub fn receive_example_from_js(val: &JsValue) {
    let example: Example = val.into_serde().unwrap();
    ...
}
```

## JavaScript Usage

In the `JsValue` that JavaScript gets, `field1` will be an `Object` (not a
JavaScript `Map`), `field2` will be a JavaScript `Array` whose members are
`Array`s of numbers, and `field3` will be an `Array` of numbers.

```js
import { send_example_to_js, receive_example_from_js } from "example";

// Get the example object from wasm.
let example = send_example_to_js();

// Add another "Vec" element to the end of the "Vec<Vec<f32>>"
example.field2.push([5, 6]);

// Send the example object back to wasm.
receive_example_from_js(example);
```

## An Alternative Approach: `serde-wasm-bindgen`

[The `serde-wasm-bindgen`
crate](https://github.com/cloudflare/serde-wasm-bindgen) serializes and
deserializes Rust structures directly to `JsValue`s, without going through
temporary JSON stringification. This approach has both advantages and
disadvantages.

The primary advantage is smaller code size: going through JSON entrenches code
to stringify and parse floating point numbers, which is not a small amount of
code. It also supports more types than JSON does, such as `Map`, `Set`, and
array buffers.

There are two primary disadvantages. The first is that it is not always
compatible with the default JSON-based serialization. The second is that it
performs more calls back and forth between JS and Wasm, which has not been fully
optimized in all engines, meaning it can sometimes be a speed
regression. However, in other cases, it is a speed up over the JSON-based
stringification, so &mdash; as always &mdash; make sure to profile your own use
cases as necessary.
