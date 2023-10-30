# `*const T` and `*mut T`

| `T` parameter | `&T` parameter | `&mut T` parameter | `T` return value | `Option<T>` parameter | `Option<T>` return value | JavaScript representation |
|:---:|:---:|:---:|:---:|:---:|:---:|:---:|
| Yes | No | No | Yes | No | No | A JavaScript number value |

## Example Rust Usage

```rust
{{#include ../../../../examples/guide-supported-types-examples/src/pointers.rs}}
```

## Example JavaScript Usage

```js
{{#include ../../../../examples/guide-supported-types-examples/pointers.js}}
```
