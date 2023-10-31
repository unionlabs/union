# `typescript_type = "Blah"`

The `typescript_type` attribute is used to specify the TypeScript type for an
imported type. This type will be used in the generated `.d.ts`.

Right now only identifiers are supported, but eventually we'd like to support
all TypeScript types.

```rust
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "Foo")]
    type Foo;
}
```
