# Communicating types to `wasm-bindgen`

The last aspect to talk about when converting Rust/JS types amongst one another
is how this information is actually communicated. The `#[wasm_bindgen]` macro is
running over the syntactical (unresolved) structure of the Rust code and is then
responsible for generating information that `wasm-bindgen` the CLI tool later
reads.

To accomplish this a slightly unconventional approach is taken. Static
information about the structure of the Rust code is serialized via JSON
(currently) to a custom section of the wasm executable. Other information, like
what the types actually are, unfortunately isn't known until later in the
compiler due to things like associated type projections and typedefs. It also
turns out that we want to convey "rich" types like `FnMut(String, Foo,
&JsValue)` to the `wasm-bindgen` CLI, and handling all this is pretty tricky!

To solve this issue the `#[wasm_bindgen]` macro generates **executable
functions** which "describe the type signature of an import or export". These
executable functions are what the `WasmDescribe` trait is all about:

```rust
pub trait WasmDescribe {
    fn describe();
}
```

While deceptively simple this trait is actually quite important. When you write,
an export like this:

```rust
#[wasm_bindgen]
fn greet(a: &str) {
    // ...
}
```

In addition to the shims we talked about above which JS generates the macro
*also* generates something like:

```
#[no_mangle]
pub extern "C" fn __wbindgen_describe_greet() {
    <dyn Fn(&str)>::describe();
}
```

Or in other words it generates invocations of `describe` functions. In doing so
the `__wbindgen_describe_greet` shim is a programmatic description of the type
layouts of an import/export. These are then executed when `wasm-bindgen` runs!
These executions rely on an import called `__wbindgen_describe` which passes one
`u32` to the host, and when called multiple times gives a `Vec<u32>`
effectively. This `Vec<u32>` can then be reparsed into an `enum Descriptor`
which fully describes a type.

All in all this is a bit roundabout but shouldn't have any impact on the
generated code or runtime at all. All these descriptor functions are pruned from
the emitted wasm file.
