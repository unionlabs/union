This is a native integration of [Serde](https://serde.rs/) with [wasm-bindgen](https://github.com/rustwasm/wasm-bindgen). It allows to convert Rust data types into native JavaScript types and vice versa.

Initially this library was created as [an alternative implementation](https://github.com/rustwasm/wasm-bindgen/issues/1258) to the JSON-based Serde support built into the `wasm-bindgen` but, as of [recently](https://github.com/rustwasm/wasm-bindgen/pull/3031) `serde-wasm-bindgen` is the officially preferred approach. It provides much smaller code size overhead than JSON, and, in most common cases, provides much faster serialization/deserialization as well.

## Usage

Copied almost verbatim from the [`wasm-bindgen` guide](https://rustwasm.github.io/wasm-bindgen/reference/arbitrary-data-with-serde.html#serializing-and-deserializing-arbitrary-data-into-and-from-jsvalue-with-serde):

### Add dependencies

To use `serde-wasm-bindgen`, you first have to add it as a dependency in your
`Cargo.toml`. You also need the `serde` crate, with the `derive` feature
enabled, to allow your types to be serialized and deserialized with Serde.

```toml
[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde-wasm-bindgen = "0.4"
```

### Derive the `Serialize` and `Deserialize` Traits

Add `#[derive(Serialize, Deserialize)]` to your type. All of your type
members must also be supported by Serde, i.e. their types must also implement
the `Serialize` and `Deserialize` traits.

Note that you don't need to use the `#[wasm_bindgen]` macro.

```rust
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct Example {
    pub field1: HashMap<u32, String>,
    pub field2: Vec<Vec<f32>>,
    pub field3: [f32; 4],
}
```

### Send it to JavaScript with `serde_wasm_bindgen::to_value`

```rust
#[wasm_bindgen]
pub fn send_example_to_js() -> Result<JsValue, JsValue> {
    let mut field1 = HashMap::new();
    field1.insert(0, String::from("ex"));

    let example = Example {
        field1,
        field2: vec![vec![1., 2.], vec![3., 4.]],
        field3: [1., 2., 3., 4.]
    };

    Ok(serde_wasm_bindgen::to_value(&example)?)
}
```

### Receive it from JavaScript with `serde_wasm_bindgen::from_value`

```rust
#[wasm_bindgen]
pub fn receive_example_from_js(val: JsValue) -> Result<(), JsValue> {
    let example: Example = serde_wasm_bindgen::from_value(val)?;
    /* …do something with `example`… */
    Ok(())
}
```

### JavaScript Usage

In the `JsValue` that JavaScript gets, `field1` will be a `Map<number, string>`,
`field2` will be an `Array<Array<number>>`, and `field3` will be an `Array<number>`.

```js
import { send_example_to_js, receive_example_from_js } from "example";

// Get the example object from wasm.
let example = send_example_to_js();

// Add another "Vec" element to the end of the "Vec<Vec<f32>>"
example.field2.push([5, 6]);

// Send the example object back to wasm.
receive_example_from_js(example);
```

## Supported Types

Note that, even though it might often be the case, by default this library doesn't attempt
to be strictly compatible with JSON, instead prioritising better
compatibility with common JavaScript idioms and representations.

If you need JSON compatibility (e.g. you want to serialize `HashMap<String, …>`
as plain objects instead of JavaScript `Map` instances), use the
[`Serializer::json_compatible()`](https://docs.rs/serde-wasm-bindgen/latest/serde_wasm_bindgen/struct.Serializer.html#method.json_compatible) preset.

By default, Rust ⬄ JavaScript conversions in `serde-wasm-bindgen` follow this table:

| Rust                              | JavaScript                           | Also supported in `from_value` |
|-----------------------------------|--------------------------------------|--------------------------------|
| `()` and `Option<T>::None`        | `undefined`                          | `null`                         |
| `bool`                            | `boolean`                            |                                |
| `f32`, `f64`                      | `number`                             |                                |
| `u8`, `i8`, …, `u32`, `i32`       | `number` in the [safe integer] range |                                |
| `u64`, `i64`, `usize`, `isize`    | `number` in the [safe integer] range | `bigint`                       |
| `u128`, `i128`                    | `bigint`                             |                                |
| `String`                          | `string`                             |                                |
| `char`                            | single-codepoint `string`            |                                |
| `Enum::Variant { … }`             | [as configured in Serde]             |                                |
| `HashMap<K, V>`, `BTreeMap`, etc. | `Map<K, V>`                          | any iterable over `[K, V]`     |
| `Struct { key1: value1, … }`      | `{ key1: value1, … }` object         |                                |
| tuple, `Vec<T>`, `HashSet`, etc.  | `T[]` array                          | any iterable over `T`          |
| [`serde_bytes`] byte buffer       | `Uint8Array`                         | `ArrayBuffer`                  |

[as configured in Serde]: https://serde.rs/enum-representations.html
[safe integer]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Number/isSafeInteger
[`serde_bytes`]: https://github.com/serde-rs/bytes

The first two columns show idiomatic representations on Rust and JavaScript sides, while the 3rd column shows which JavaScript values
are additionally supported when deserializing from JavaScript to the Rust type.

### Serializer configuration options

You can customize serialization from Rust to JavaScript by setting the following options on the [`Serializer::new()`](https://docs.rs/serde-wasm-bindgen/latest/serde_wasm_bindgen/struct.Serializer.html) instance:

- `.serialize_missing_as_null(true)`: Serialize `()`, unit structs and `Option::None` to `null` instead of `undefined`.
- `.serialize_maps_as_objects(true)`: Serialize maps into plain JavaScript objects instead of ES2015 Maps. false by default.
- `.serialize_large_number_types_as_bigints(true)`: Serialize `u64`, `i64`, `usize` and `isize` to `bigint`s instead of attempting to fit them into the [safe integer] `number` or failing.

You can also use the `Serializer::json_compatible()` preset to create a JSON compatible serializer. It enables `serialize_missing_as_null` and `serialize_maps_as_objects` under the hood.

## License

Licensed under the MIT license. See the
[LICENSE](https://github.com/cloudflare/serde-wasm-bindgen/blob/master/LICENSE)
file for details.
