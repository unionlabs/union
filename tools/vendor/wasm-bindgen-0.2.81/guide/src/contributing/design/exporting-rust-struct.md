# Exporting a struct to JS

So far we've covered JS objects, importing functions, and exporting functions.
This has given us quite a rich base to build on so far, and that's great! We
sometimes, though, want to go even further and define a JS `class` in Rust. Or
in other words, we want to expose an object with methods from Rust to JS rather
than just importing/exporting free functions.

The `#[wasm_bindgen]` attribute can annotate both a `struct` and `impl` blocks
to allow:

```rust
#[wasm_bindgen]
pub struct Foo {
    internal: i32,
}

#[wasm_bindgen]
impl Foo {
    #[wasm_bindgen(constructor)]
    pub fn new(val: i32) -> Foo {
        Foo { internal: val }
    }

    pub fn get(&self) -> i32 {
        self.internal
    }

    pub fn set(&mut self, val: i32) {
        self.internal = val;
    }
}
```

This is a typical Rust `struct` definition for a type with a constructor and a
few methods. Annotating the struct with `#[wasm_bindgen]` means that we'll
generate necessary trait impls to convert this type to/from the JS boundary. The
annotated `impl` block here means that the functions inside will also be made
available to JS through generated shims. If we take a look at the generated JS
code for this we'll see:

```js
import * as wasm from './js_hello_world_bg';

export class Foo {
    static __construct(ptr) {
        return new Foo(ptr);
    }

    constructor(ptr) {
        this.ptr = ptr;
    }

    free() {
        const ptr = this.ptr;
        this.ptr = 0;
        wasm.__wbg_foo_free(ptr);
    }

    static new(arg0) {
        const ret = wasm.foo_new(arg0);
        return Foo.__construct(ret)
    }

    get() {
        const ret = wasm.foo_get(this.ptr);
        return ret;
    }

    set(arg0) {
        const ret = wasm.foo_set(this.ptr, arg0);
        return ret;
    }
}
```

That's actually not much! We can see here though how we've translated from Rust
to JS:

* Associated functions in Rust (those without `self`) turn into `static`
  functions in JS.
* Methods in Rust turn into methods in wasm.
* Manual memory management is exposed in JS as well. The `free` function is
  required to be invoked to deallocate resources on the Rust side of things.

To be able to use `new Foo()`, you'd need to annotate `new` as `#[wasm_bindgen(constructor)]`.

One important aspect to note here, though, is that once `free` is called the JS
object is "neutered" in that its internal pointer is nulled out. This means that
future usage of this object should trigger a panic in Rust.

The real trickery with these bindings ends up happening in Rust, however, so
let's take a look at that.

```rust
// original input to `#[wasm_bindgen]` omitted ...

#[export_name = "foo_new"]
pub extern "C" fn __wasm_bindgen_generated_Foo_new(arg0: i32) -> u32
    let ret = Foo::new(arg0);
    Box::into_raw(Box::new(WasmRefCell::new(ret))) as u32
}

#[export_name = "foo_get"]
pub extern "C" fn __wasm_bindgen_generated_Foo_get(me: u32) -> i32 {
    let me = me as *mut WasmRefCell<Foo>;
    wasm_bindgen::__rt::assert_not_null(me);
    let me = unsafe { &*me };
    return me.borrow().get();
}

#[export_name = "foo_set"]
pub extern "C" fn __wasm_bindgen_generated_Foo_set(me: u32, arg1: i32) {
    let me = me as *mut WasmRefCell<Foo>;
    wasm_bindgen::__rt::assert_not_null(me);
    let me = unsafe { &*me };
    me.borrow_mut().set(arg1);
}

#[no_mangle]
pub unsafe extern "C" fn __wbindgen_foo_free(me: u32) {
    let me = me as *mut WasmRefCell<Foo>;
    wasm_bindgen::__rt::assert_not_null(me);
    (*me).borrow_mut(); // ensure no active borrows
    drop(Box::from_raw(me));
}
```

As with before this is cleaned up from the actual output but it's the same idea
as to what's going on! Here we can see a shim for each function as well as a
shim for deallocating an instance of `Foo`. Recall that the only valid wasm
types today are numbers, so we're required to shoehorn all of `Foo` into a
`u32`, which is currently done via `Box` (like `std::unique_ptr` in C++).
Note, though, that there's an extra layer here, `WasmRefCell`. This type is the
same as [`RefCell`] and can be mostly glossed over.

The purpose for this type, if you're interested though, is to uphold Rust's
guarantees about aliasing in a world where aliasing is rampant (JS).
Specifically the `&Foo` type means that there can be as much aliasing as you'd
like, but crucially `&mut Foo` means that it is the sole pointer to the data
(no other `&Foo` to the same instance exists). The [`RefCell`] type in libstd
is a way of dynamically enforcing this at runtime (as opposed to compile time
where it usually happens). Baking in `WasmRefCell` is the same idea here,
adding runtime checks for aliasing which are typically happening at compile
time. This is currently a Rust-specific feature which isn't actually in the
`wasm-bindgen` tool itself, it's just in the Rust-generated code (aka the
`#[wasm_bindgen]` attribute).

[`RefCell`]: https://doc.rust-lang.org/std/cell/struct.RefCell.html
