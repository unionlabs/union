# Number Slices: `[u8]`, `[i8]`, `[u16]`, `[i16]`, `[u32]`, `[i32]`, `[u64]`, `[i64]`, `[f32]`, and `[f64]`

| `T` parameter | `&T` parameter | `&mut T` parameter | `T` return value | `Option<&T>` parameter | `Option<T>` return value | JavaScript representation |
|:---:|:---:|:---:|:---:|:---:|:---:|:---:|
| No | Yes | Yes | No | No | No | A JavaScript `TypedArray` view of the Wasm memory for the boxed slice of the appropriate type (`Int32Array`, `Uint8Array`, etc) |

## Example Rust Usage

```rust
{{#include ../../../../examples/guide-supported-types-examples/src/number_slices.rs}}
```

## Example JavaScript Usage

```js
{{#include ../../../../examples/guide-supported-types-examples/number_slices.js}}
```
