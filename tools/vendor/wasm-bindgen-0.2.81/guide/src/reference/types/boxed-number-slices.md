# Boxed Number Slices: `Box<[u8]>`, `Box<[i8]>`, `Box<[u16]>`, `Box<[i16]>`, `Box<[u32]>`, `Box<[i32]>`, `Box<[u64]>`, `Box<[i64]>`, `Box<[f32]>`, and `Box<[f64]>`

| `T` parameter | `&T` parameter | `&mut T` parameter | `T` return value | `Option<T>` parameter | `Option<T>` return value | JavaScript representation |
|:---:|:---:|:---:|:---:|:---:|:---:|:---:|
| Yes | No | No | Yes | Yes | Yes | A JavaScript `TypedArray` of the appropriate type (`Int32Array`, `Uint8Array`, etc...) |

Note that the contents of the slice are copied into the JavaScript `TypedArray`
from the Wasm linear memory when returning a boxed slice to JavaScript, and vice
versa when receiving a JavaScript `TypedArray` as a boxed slice in Rust.

## Example Rust Usage

```rust
{{#include ../../../../examples/guide-supported-types-examples/src/boxed_number_slices.rs}}
```

## Example JavaScript Usage

```js
{{#include ../../../../examples/guide-supported-types-examples/boxed_number_slices.js}}
```
