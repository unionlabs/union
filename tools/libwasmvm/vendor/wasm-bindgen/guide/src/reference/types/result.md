# `Result<T, E>`

| `T` parameter | `&T` parameter | `&mut T` parameter | `T` return value | `Option<T>` parameter | `Option<T>` return value | JavaScript representation |
|:---:|:---:|:---:|:---:|:---:|:---:|:---:|
| No | No | No | Yes | No | No | Same as `T`, or an exception |

The `Result` type can be returned from functions exported to JS as well as
closures in Rust. The `Ok` type must be able to be converted to JS, and the
`Err` type must implement `Into<JsValue>`. Whenever `Ok(val)` is encountered
it's converted to JS and handed off, and whenever `Err(error)` is encountered
an exception is thrown in JS with `error`.

You can use `Result` to enable handling of JS exceptions with `?` in Rust,
naturally propagating it upwards to the wasm boundary. Furthermore you can also
return custom types in Rust so long as they're all convertible to `JsValue`.

Note that if you import a JS function with `Result` you need
`#[wasm_bindgen(catch)]` to be annotated on the import (unlike exported
functions, which require no extra annotation). This may not be necessary in the
future though and it may work "as is"!.
