# `extends = Class`

The `extends` attribute can be used to say that an imported type extends (in the
JS class hierarchy sense) another type. This will generate `AsRef`, `AsMut`, and
`From` impls for converting a type into another given that we statically know
the inheritance hierarchy:

```rust
#[wasm_bindgen]
extern "C" {
    type Foo;

    #[wasm_bindgen(extends = Foo)]
    type Bar;
}

let x: &Bar = ...;
let y: &Foo = x.as_ref(); // zero cost cast
```

The trait implementations generated for the above block are:

```rust
impl From<Bar> for Foo { ... }
impl AsRef<Foo> for Bar { ... }
impl AsMut<Foo> for Bar { ... }
```


The `extends = ...` attribute can be specified multiple times for longer
inheritance chains, and `AsRef` and such impls will be generated for each of
the types.

```rust
#[wasm_bindgen]
extern "C" {
    type Foo;

    #[wasm_bindgen(extends = Foo)]
    type Bar;

    #[wasm_bindgen(extends = Foo, extends = Bar)]
    type Baz;
}

let x: &Baz = ...;
let y1: &Bar = x.as_ref();
let y2: &Foo = y1.as_ref();
```
