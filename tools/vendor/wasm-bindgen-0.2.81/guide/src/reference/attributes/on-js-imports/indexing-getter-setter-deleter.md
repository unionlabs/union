# `indexing_getter`, `indexing_setter`, and `indexing_deleter`

These three attributes indicate that a method is an dynamically intercepted
getter, setter, or deleter on the receiver object itself, rather than a direct
access of the receiver's properties. It is equivalent calling the Proxy handler
for the `obj[prop]` operation with some dynamic `prop` variable in JavaScript,
rather than a normal static property access like `obj.prop` on a normal
JavaScript `Object`.

This is useful for binding to `Proxy`s and some builtin DOM types that
dynamically intercept property accesses.

* `indexing_getter` corresponds to `obj[prop]` operation in JavaScript. The
  function annotated must have a `this` receiver parameter, a single parameter
  that is used for indexing into the receiver (`prop`), and a return type.

* `indexing_setter` corresponds to the `obj[prop] = val` operation in
  JavaScript. The function annotated must have a `this` receiver parameter, a
  parameter for indexing into the receiver (`prop`), and a value parameter
  (`val`).

* `indexing_deleter` corresponds to `delete obj[prop]` operation in
  JavaScript. The function annotated must have a `this` receiver and a single
  parameter for indexing into the receiver (`prop`).

These must always be used in conjunction with the `structural` and `method`
flags.

For example, consider this JavaScript snippet that uses `Proxy`:

```js
const foo = new Proxy({}, {
    get(obj, prop) {
        return prop in obj ? obj[prop] : prop.length;
    },
    set(obj, prop, value) {
        obj[prop] = value;
    },
    deleteProperty(obj, prop) {
        delete obj[prop];
    },
});

foo.ten;
// 3

foo.ten = 10;
foo.ten;
// 10

delete foo.ten;
foo.ten;
// 3
```

To bind that in `wasm-bindgen` in Rust, we would use the `indexing_*` attributes
on methods:

```rust
#[wasm_bindgen]
extern "C" {
    type Foo;
    static foo: Foo;

    #[wasm_bindgen(method, structural, indexing_getter)]
    fn get(this: &Foo, prop: &str) -> u32;

    #[wasm_bindgen(method, structural, indexing_setter)]
    fn set(this: &Foo, prop: &str, val: u32);

    #[wasm_bindgen(method, structural, indexing_deleter)]
    fn delete(this: &Foo, prop: &str);
}

assert_eq!(foo.get("ten"), 3);

foo.set("ten", 10);
assert_eq!(foo.get("ten"), 10);

foo.delete("ten");
assert_eq!(foo.get("ten"), 3);
```
