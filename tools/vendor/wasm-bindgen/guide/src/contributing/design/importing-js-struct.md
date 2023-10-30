# Importing a class from JS

Just like with functions after we've started exporting we'll also want to
import! Now that we've exported a `class` to JS we'll want to also be able to
import classes in Rust as well to invoke methods and such. Since JS classes are
in general just JS objects the bindings here will look pretty similar to the JS
object bindings describe above.

As usual though, let's dive into an example!

```rust
#[wasm_bindgen(module = "./bar")]
extern "C" {
    type Bar;

    #[wasm_bindgen(constructor)]
    fn new(arg: i32) -> Bar;

    #[wasm_bindgen(js_namespace = Bar)]
    fn another_function() -> i32;

    #[wasm_bindgen(method)]
    fn get(this: &Bar) -> i32;

    #[wasm_bindgen(method)]
    fn set(this: &Bar, val: i32);

    #[wasm_bindgen(method, getter)]
    fn property(this: &Bar) -> i32;

    #[wasm_bindgen(method, setter)]
    fn set_property(this: &Bar, val: i32);
}

fn run() {
    let bar = Bar::new(Bar::another_function());
    let x = bar.get();
    bar.set(x + 3);

    bar.set_property(bar.property() + 6);
}
```

Unlike our previous imports, this one's a bit more chatty! Remember that one of
the goals of `wasm-bindgen` is to use native Rust syntax wherever possible, so
this is mostly intended to use the `#[wasm_bindgen]` attribute to interpret
what's written down in Rust. Now there's a few attribute annotations here, so
let's go through one-by-one:

* `#[wasm_bindgen(module = "./bar")]` - seen before with imports this is declare
  where all the subsequent functionality is imported from. For example the `Bar`
  type is going to be imported from the `./bar` module.
* `type Bar` - this is a declaration of JS class as a new type in Rust. This
  means that a new type `Bar` is generated which is "opaque" but is represented
  as internally containing a `JsValue`. We'll see more on this later.
* `#[wasm_bindgen(constructor)]` - this indicates that the binding's name isn't
  actually used in JS but rather translates to `new Bar()`. The return value of
  this function must be a bare type, like `Bar`.
* `#[wasm_bindgen(js_namespace = Bar)]` - this attribute indicates that the
  function declaration is namespaced through the `Bar` class in JS.
* `#[wasm_bindgen(static_method_of = SomeJsClass)]` - this attribute is similar
  to `js_namespace`, but instead of producing a free function, produces a static
  method of `SomeJsClass`.
* `#[wasm_bindgen(method)]` - and finally, this attribute indicates that a
  method call is going to happen. The first argument must be a JS struct, like
  `Bar`, and the call in JS looks like `Bar.prototype.set.call(...)`.

With all that in mind, let's take a look at the JS generated.

```js
import * as wasm from './foo_bg';

import { Bar } from './bar';

// other support functions omitted...

export function __wbg_s_Bar_new() {
  return addHeapObject(new Bar());
}

const another_function_shim = Bar.another_function;
export function __wbg_s_Bar_another_function() {
  return another_function_shim();
}

const get_shim = Bar.prototype.get;
export function __wbg_s_Bar_get(ptr) {
  return shim.call(getObject(ptr));
}

const set_shim = Bar.prototype.set;
export function __wbg_s_Bar_set(ptr, arg0) {
  set_shim.call(getObject(ptr), arg0)
}

const property_shim = Object.getOwnPropertyDescriptor(Bar.prototype, 'property').get;
export function __wbg_s_Bar_property(ptr) {
  return property_shim.call(getObject(ptr));
}

const set_property_shim = Object.getOwnPropertyDescriptor(Bar.prototype, 'property').set;
export function __wbg_s_Bar_set_property(ptr, arg0) {
  set_property_shim.call(getObject(ptr), arg0)
}
```

Like when importing functions from JS we can see a bunch of shims are generated
for all the relevant functions. The `new` static function has the
`#[wasm_bindgen(constructor)]` attribute which means that instead of any
particular method it should actually invoke the `new` constructor instead (as
we see here). The static function `another_function`, however, is dispatched as
`Bar.another_function`.

The `get` and `set` functions are methods so they go through `Bar.prototype`,
and otherwise their first argument is implicitly the JS object itself which is
loaded through `getObject` like we saw earlier.

Some real meat starts to show up though on the Rust side of things, so let's
take a look:

```rust
pub struct Bar {
    obj: JsValue,
}

impl Bar {
    fn new() -> Bar {
        extern "C" {
            fn __wbg_s_Bar_new() -> u32;
        }
        unsafe {
            let ret = __wbg_s_Bar_new();
            Bar { obj: JsValue::__from_idx(ret) }
        }
    }

    fn another_function() -> i32 {
        extern "C" {
            fn __wbg_s_Bar_another_function() -> i32;
        }
        unsafe {
            __wbg_s_Bar_another_function()
        }
    }

    fn get(&self) -> i32 {
        extern "C" {
            fn __wbg_s_Bar_get(ptr: u32) -> i32;
        }
        unsafe {
            let ptr = self.obj.__get_idx();
            let ret = __wbg_s_Bar_get(ptr);
            return ret
        }
    }

    fn set(&self, val: i32) {
        extern "C" {
            fn __wbg_s_Bar_set(ptr: u32, val: i32);
        }
        unsafe {
            let ptr = self.obj.__get_idx();
            __wbg_s_Bar_set(ptr, val);
        }
    }

    fn property(&self) -> i32 {
        extern "C" {
            fn __wbg_s_Bar_property(ptr: u32) -> i32;
        }
        unsafe {
            let ptr = self.obj.__get_idx();
            let ret = __wbg_s_Bar_property(ptr);
            return ret
        }
    }

    fn set_property(&self, val: i32) {
        extern "C" {
            fn __wbg_s_Bar_set_property(ptr: u32, val: i32);
        }
        unsafe {
            let ptr = self.obj.__get_idx();
            __wbg_s_Bar_set_property(ptr, val);
        }
    }
}

impl WasmBoundary for Bar {
    // ...
}

impl ToRefWasmBoundary for Bar {
    // ...
}
```

In Rust we're seeing that a new type, `Bar`, is generated for this import of a
class. The type `Bar` internally contains a `JsValue` as an instance of `Bar`
is meant to represent a JS object stored in our module's stack/slab. This then
works mostly the same way that we saw JS objects work in the beginning.

When calling `Bar::new` we'll get an index back which is wrapped up in `Bar`
(which is itself just a `u32` in memory when stripped down). Each function then
passes the index as the first argument and otherwise forwards everything along
in Rust.
