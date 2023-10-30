# `typescript_custom_section`

When added to a `const` `&'static str`, it will append the contents of the
string to the `.d.ts` file exported by `wasm-bindgen-cli` (when the
`--typescript` flag is enabled).

```rust
#[wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = r#"

export type Coords = { "latitude": number, "longitude": number, }; 

"#;
```

The primary target for this feature is for code generation. For example, you
can author a macro that allows you to export a TypeScript definition alongside
the definition of a struct or Rust type.

```rust
#[derive(MyTypescriptExport)]
struct Coords {
    latitude: u32,
    longitude: u32,
}
```

The proc_derive_macro "MyTypescriptExport" can export its own
`#[wasm_bindgen(typescript_custom_section)]` section, which would then be
picked up by wasm-bindgen-cli. This would be equivalent to the contents of
the TS_APPEND_CONTENT string in the first example.

This feature allows plain data objects to be typechecked in Rust and in
TypeScript by outputting a type definition generated at compile time.
