# `structural`

> **Note**: As of [RFC 5] this attribute is the default for all imported
> functions. This attribute is largely ignored today and is only retained for
> backwards compatibility and learning purposes.
>
> The inverse of this attribute, [the `final`
> attribute](final.html) is more functionally interesting than
> `structural` (as `structural` is simply the default)

[RFC 5]: https://rustwasm.github.io/rfcs/005-structural-and-deref.html

The `structural` flag can be added to `method` annotations, indicating that the
method being accessed (or property with getters/setters) should be accessed in a
structural, duck-type-y fashion. Rather than walking the constructor's prototype
chain once at load time and caching the property result, the prototype chain is
dynamically walked on every access.

```rust
#[wasm_bindgen]
extern "C" {
    type Duck;

    #[wasm_bindgen(method, structural)]
    fn quack(this: &Duck);

    #[wasm_bindgen(method, getter, structural)]
    fn is_swimming(this: &Duck) -> bool;
}
```

The constructor for the type here, `Duck`, is not required to exist in
JavaScript (it's not referenced).  Instead `wasm-bindgen` will generate shims
that will access the passed in JavaScript value's `quack` method or its
`is_swimming` property.

```js
// Without `structural`, get the method directly off the prototype at load time:
const Duck_prototype_quack = Duck.prototype.quack;
function quack(duck) {
  Duck_prototype_quack.call(duck);
}

// With `structural`, walk the prototype chain on every access:
function quack(duck) {
  duck.quack();
}
```
