# `skip_typescript`

By default, Rust exports exposed to JavaScript will generate TypeScript definitions (unless `--no-typescript` is used). The `skip_typescript` attribute can be used to disable type generation per function, enum, struct, or field. For example:

```rust
#[wasm_bindgen(skip_typescript)]
pub enum MyHiddenEnum {
    One,
    Two,
    Three
}

#[wasm_bindgen]
pub struct MyPoint {
    pub x: u32,

    #[wasm_bindgen(skip_typescript)]
    pub y: u32,
}

#[wasm_bindgen]
impl MyPoint {

    #[wasm_bindgen(skip_typescript)]
    pub fn stringify(&self) -> String {
        format!("({}, {})", self.x, self.y)
    }
}
```

Will generate the following `.d.ts` file:

```ts
/* tslint:disable */
/* eslint-disable */
export class MyPoint {
  free(): void;
  x: number;
}
```

When combined with [the `typescript_custom_section` attribute](typescript_custom_section.html), this can be used to manually specify more specific function types instead of using the generated definitions.
