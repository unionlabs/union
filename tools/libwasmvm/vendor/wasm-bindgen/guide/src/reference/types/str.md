# `str`

| `T` parameter | `&T` parameter | `&mut T` parameter | `T` return value | `Option<T>` parameter | `Option<T>` return value | JavaScript representation |
|:---:|:---:|:---:|:---:|:---:|:---:|:---:|
| No | Yes | No | No | No | No | JavaScript string value |

Copies the string's contents back and forth between the JavaScript
garbage-collected heap and the Wasm linear memory with `TextDecoder` and
`TextEncoder`. If you don't want to perform this copy, and would rather work
with handles to JavaScript string values, use the `js_sys::JsString` type.

## Example Rust Usage

```rust
{{#include ../../../../examples/guide-supported-types-examples/src/str.rs}}
```

## Example JavaScript Usage

```js
{{#include ../../../../examples/guide-supported-types-examples/str.js}}
```

## UTF-16 vs UTF-8

Strings in JavaScript are encoded as UTF-16, but with one major exception: they
can contain unpaired surrogates. For some Unicode characters UTF-16 uses two
16-bit values.  These are called "surrogate pairs" because they always come in
pairs. In JavaScript, it is possible for these surrogate pairs to be missing the
other half, creating an "unpaired surrogate".

When passing a string from JavaScript to Rust, it uses the `TextEncoder` API to
convert from UTF-16 to UTF-8. This is normally perfectly fine... unless there
are unpaired surrogates. In that case it will replace the unpaired surrogates
with U+FFFD (�, the replacement character). That means the string in Rust is
now different from the string in JavaScript!

If you want to guarantee that the Rust string is the same as the JavaScript
string, you should instead use `js_sys::JsString` (which keeps the string in
JavaScript and doesn't copy it into Rust).

If you want to access the raw value of a JS string, you can use `JsString::iter`,
which returns an `Iterator<Item = u16>`. This perfectly preserves everything
(including unpaired surrogates), but it does not do any encoding (so you
have to do that yourself!).

If you simply want to ignore strings which contain unpaired surrogates, you can
use `JsString::is_valid_utf16` to test whether the string contains unpaired
surrogates or not.
