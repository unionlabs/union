# Exporting a function to JS

Alright now that we've got a good grasp on JS objects and how they're working,
let's take a look at another feature of `wasm-bindgen`: exporting functionality
with types that are richer than just numbers.

The basic idea around exporting functionality with more flavorful types is that
the wasm exports won't actually be called directly. Instead the generated
`foo.js` module will have shims for all exported functions in the wasm module.

The most interesting conversion here happens with strings so let's take a look
at that.

```rust
#[wasm_bindgen]
pub fn greet(a: &str) -> String {
    format!("Hello, {}!", a)
}
```

Here we'd like to define an ES module that looks like

```ts
// foo.d.ts
export function greet(a: string): string;
```

To see what's going on, let's take a look at the generated shim

```js
import * as wasm from './foo_bg';

function passStringToWasm(arg) {
  const buf = new TextEncoder('utf-8').encode(arg);
  const len = buf.length;
  const ptr = wasm.__wbindgen_malloc(len);
  let array = new Uint8Array(wasm.memory.buffer);
  array.set(buf, ptr);
  return [ptr, len];
}

function getStringFromWasm(ptr, len) {
  const mem = new Uint8Array(wasm.memory.buffer);
  const slice = mem.slice(ptr, ptr + len);
  const ret = new TextDecoder('utf-8').decode(slice);
  return ret;
}

export function greet(arg0) {
  const [ptr0, len0] = passStringToWasm(arg0);
  try {
    const ret = wasm.greet(ptr0, len0);
    const ptr = wasm.__wbindgen_boxed_str_ptr(ret);
    const len = wasm.__wbindgen_boxed_str_len(ret);
    const realRet = getStringFromWasm(ptr, len);
    wasm.__wbindgen_boxed_str_free(ret);
    return realRet;
  } finally {
    wasm.__wbindgen_free(ptr0, len0);
  }
}
```

Phew, that's quite a lot! We can sort of see though if we look closely what's
happening:

* Strings are passed to wasm via two arguments, a pointer and a length. Right
  now we have to copy the string onto the wasm heap which means we'll be using
  `TextEncoder` to actually do the encoding. Once this is done we use an
  internal function in `wasm-bindgen` to allocate space for the string to go,
  and then we'll pass that ptr/length to wasm later on.

* Returning strings from wasm is a little tricky as we need to return a ptr/len
  pair, but wasm currently only supports one return value (multiple return values
  [is being standardized](https://github.com/WebAssembly/design/issues/1146)).
  To work around this in the meantime, we're actually returning a pointer to a
  ptr/len pair, and then using functions to access the various fields.

* Some cleanup ends up happening in wasm. The `__wbindgen_boxed_str_free`
  function is used to free the return value of `greet` after it's been decoded
  onto the JS heap (using `TextDecoder`). The `__wbindgen_free` is then used to
  free the space we allocated to pass the string argument once the function call
  is done.

Next let's take a look at the Rust side of things as well. Here we'll be looking
at a mostly abbreviated and/or "simplified" in the sense of this is what it
compiles down to:

```rust
pub extern "C" fn greet(a: &str) -> String {
    format!("Hello, {}!", a)
}

#[export_name = "greet"]
pub extern "C" fn __wasm_bindgen_generated_greet(
    arg0_ptr: *const u8,
    arg0_len: usize,
) -> *mut String {
    let arg0 = unsafe {
        let slice = ::std::slice::from_raw_parts(arg0_ptr, arg0_len);
        ::std::str::from_utf8_unchecked(slice)
    };
    let _ret = greet(arg0);
    Box::into_raw(Box::new(_ret))
}
```

Here we can see again that our `greet` function is unmodified and has a wrapper
to call it. This wrapper will take the ptr/len argument and convert it to a
string slice, while the return value is boxed up into just a pointer and is
then returned up to was for reading via the `__wbindgen_boxed_str_*` functions.

So in general exporting a function involves a shim both in JS and in Rust with
each side translating to or from wasm arguments to the native types of each
language. The `wasm-bindgen` tool manages hooking up all these shims while the
`#[wasm_bindgen]` macro takes care of the Rust shim as well.

Most arguments have a relatively clear way to convert them, bit if you've got
any questions just let me know!

