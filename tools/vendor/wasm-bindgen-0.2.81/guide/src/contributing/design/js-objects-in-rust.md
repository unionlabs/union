# Polyfill for "JS objects in wasm"

One of the main goals of `wasm-bindgen` is to allow working with and passing
around JS objects in wasm, but that's not allowed today! While indeed true,
that's where the polyfill comes in.

The question here is how we shoehorn JS objects into a `u32` for wasm to use.
The current strategy for this approach is to maintain a module-local variable
in the generated `foo.js` file: a `heap`.

### Temporary JS objects on the "stack"

The first slots in the `heap` in `foo.js` are considered a stack. This stack,
like typical program execution stacks, grows down. JS objects are pushed on the
bottom of the stack, and their index in the stack is the identifier that's passed
to wasm. A stack pointer is maintained to figure out where the next item is
pushed.

JS objects are then only removed from the bottom of the stack as well. Removal
is simply storing null then incrementing a counter.  Because of the "stack-y"
nature of this scheme it only works for when wasm doesn't hold onto a JS object
(aka it only gets a "reference" in Rust parlance).

Let's take a look at an example.

```rust
// foo.rs
#[wasm_bindgen]
pub fn foo(a: &JsValue) {
    // ...
}
```

Here we're using the special `JsValue` type from the `wasm-bindgen` library
itself. Our exported function, `foo`, takes a *reference* to an object. This
notably means that it can't persist the object past the lifetime of this
function call.

Now what we actually want to generate is a JS module that looks like (in
TypeScript parlance)

```ts
// foo.d.ts
export function foo(a: any);
```

and what we actually generate looks something like:

```js
// foo.js
import * as wasm from './foo_bg';

const heap = new Array(32);
heap.push(undefined, null, true, false);
let stack_pointer = 32;

function addBorrowedObject(obj) {
  stack_pointer -= 1;
  heap[stack_pointer] = obj;
  return stack_pointer;
}

export function foo(arg0) {
  const idx0 = addBorrowedObject(arg0);
  try {
    wasm.foo(idx0);
  } finally {
    heap[stack_pointer++] = undefined;
  }
}
```

Here we can see a few notable points of action:

* The wasm file was renamed to `foo_bg.wasm`, and we can see how the JS module
  generated here is importing from the wasm file.
* Next we can see our `heap` module variable which is to store all JS values
  reference-able from wasm.
* Our exported function `foo`, takes an arbitrary argument, `arg0`, which is
  converted to an index with the `addBorrowedObject` object function. The index
  is then passed to wasm so wasm can operate with it.
* Finally, we have a `finally` which frees the stack slot as it's no longer
  used, popping the value that was pushed at the start of the function.

It's also helpful to dig into the Rust side of things to see what's going on
there! Let's take a look at the code that `#[wasm_bindgen]` generates in Rust:

```rust
// what the user wrote
pub fn foo(a: &JsValue) {
    // ...
}

#[export_name = "foo"]
pub extern "C" fn __wasm_bindgen_generated_foo(arg0: u32) {
    let arg0 = unsafe {
        ManuallyDrop::new(JsValue::__from_idx(arg0))
    };
    let arg0 = &*arg0;
    foo(arg0);
}
```

And as with the JS, the notable points here are:

* The original function, `foo`, is unmodified in the output
* A generated function here (with a unique name) is the one that's actually
  exported from the wasm module
* Our generated function takes an integer argument (our index) and then wraps it
  in a `JsValue`. There's some trickery here that's not worth going into just
  yet, but we'll see in a bit what's happening under the hood.

### Long-lived JS objects

The above strategy is useful when JS objects are only temporarily used in Rust,
for example only during one function call. Sometimes, though, objects may have a
dynamic lifetime or otherwise need to be stored on Rust's heap. To cope with
this there's a second half of management of JS objects, naturally corresponding
to the other side of the JS `heap` array.

JS Objects passed to wasm that are not references are assumed to have a dynamic
lifetime inside of the wasm module. As a result the strict push/pop of the stack
won't work and we need more permanent storage for the JS objects. To cope with
this we build our own "slab allocator" of sorts.

A picture (or code) is worth a thousand words so let's show what happens with an
example.

```rust
// foo.rs
#[wasm_bindgen]
pub fn foo(a: JsValue) {
    // ...
}
```

Note that the `&` is missing in front of the `JsValue` we had before, and in
Rust parlance this means it's taking ownership of the JS value. The exported ES
module interface is the same as before, but the ownership mechanics are slightly
different. Let's see the generated JS's slab in action:

```js
import * as wasm from './foo_bg'; // imports from wasm file

const heap = new Array(32);
heap.push(undefined, null, true, false);
let heap_next = 36;

function addHeapObject(obj) {
  if (heap_next === heap.length)
    heap.push(heap.length + 1);
  const idx = heap_next;
  heap_next = heap[idx];
  heap[idx] = obj;
  return idx;
}

export function foo(arg0) {
  const idx0 = addHeapObject(arg0);
  wasm.foo(idx0);
}

export function __wbindgen_object_drop_ref(idx) {
  heap[idx ] = heap_next;
  heap_next = idx;
}
```

Unlike before we're now calling `addHeapObject` on the argument to `foo` rather
than `addBorrowedObject`. This function will use `heap` and `heap_next` as a
slab allocator to acquire a slot to store the object, placing a structure there
once it's found. Note that this is going on the right-half of the array, unlike
the stack which resides on the left half. This discipline mirrors the stack/heap
in normal programs, roughly.

Another curious aspect of this generated module is the
`__wbindgen_object_drop_ref` function. This is one that's actually imported to
wasm rather than used in this module! This function is used to signal the end of
the lifetime of a `JsValue` in Rust, or in other words when it goes out of
scope. Otherwise though this function is largely just a general "slab free"
implementation.

And finally, let's take a look at the Rust generated again too:

```rust
// what the user wrote
pub fn foo(a: JsValue) {
    // ...
}

#[export_name = "foo"]
pub extern "C" fn __wasm_bindgen_generated_foo(arg0: u32) {
    let arg0 = unsafe {
        JsValue::__from_idx(arg0)
    };
    foo(arg0);
}
```

Ah that looks much more familiar! Not much interesting is happening here, so
let's move on to...

### Anatomy of `JsValue`

Currently the `JsValue` struct is actually quite simple in Rust, it's:

```rust
pub struct JsValue {
    idx: u32,
}

// "private" constructors

impl Drop for JsValue {
    fn drop(&mut self) {
        unsafe {
            __wbindgen_object_drop_ref(self.idx);
        }
    }
}
```

Or in other words it's a newtype wrapper around a `u32`, the index that we're
passed from wasm. The destructor here is where the `__wbindgen_object_drop_ref`
function is called to relinquish our reference count of the JS object, freeing
up our slot in the `slab` that we saw above.

If you'll recall as well, when we took `&JsValue` above we generated a wrapper
of `ManuallyDrop` around the local binding, and that's because we wanted to
avoid invoking this destructor when the object comes from the stack.

### Working with `heap` in reality

The above explanations are pretty close to what happens today, but in reality
there's a few differences especially around handling constant values like
`undefined`, `null`, etc. Be sure to check out the actual generated JS and the
generation code for the full details!
