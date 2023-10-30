# `String`

| `T` parameter | `&T` parameter | `&mut T` parameter | `T` return value | `Option<T>` parameter | `Option<T>` return value | JavaScript representation |
|:---:|:---:|:---:|:---:|:---:|:---:|:---:|
| Yes | No | No | Yes | Yes | Yes | JavaScript string value |

Copies the string's contents back and forth between the JavaScript
garbage-collected heap and the Wasm linear memory with `TextDecoder` and
`TextEncoder`

> **Note**: Be sure to check out the [documentation for `str`](str.html) to
> learn about some caveats when working with strings between JS and Rust.

## Example Rust Usage

```rust
{{#include ../../../../examples/guide-supported-types-examples/src/string.rs}}
```

## Example JavaScript Usage

```js
{{#include ../../../../examples/guide-supported-types-examples/string.js}}
```
