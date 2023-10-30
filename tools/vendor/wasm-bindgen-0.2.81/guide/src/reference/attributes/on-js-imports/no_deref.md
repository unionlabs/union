# `no_deref`

The `no_deref` attribute can be used to say that no `Deref` impl should be
generated for an imported type. If this attribute is not present, a `Deref` impl
will be generated with a `Target` of the type's first `extends` attribute, or
`Target = JsValue` if there are no `extends` attributes.

```rust
#[wasm_bindgen]
extern "C" {
    type Foo;

    #[wasm_bindgen(method)]
    fn baz(this: &Foo)

    #[wasm_bindgen(extends = Foo, no_deref)]
    type Bar;
}

fn do_stuff(bar: &Bar) {
    bar.baz() // Does not compile
}

```
