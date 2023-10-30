# `js_namespace = blah`

This attribute indicates that the JavaScript type is accessed through the given
namespace. For example, the `WebAssembly.Module` APIs are all accessed through
the `WebAssembly` namespace. `js_namespace` can be applied to any import
(function or type) and whenever the generated JavaScript attempts to reference a
name (like a class or function name) it'll be accessed through this namespace.

```rust
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
    
    type Foo;
    #[wasm_bindgen(constructor, js_namespace = Bar)]
    fn new() -> Foo;
}

log("hello, console!");
Foo::new();
```

This is an example of how to bind namespaced items in Rust. The `log` and `Foo::new` functions will
be available in the Rust module and will be invoked as `console.log` and `new Bar.Foo` in
JavaScript.

It is also possible to access the JavaScript object under the nested namespace.
`js_namespace` also accepts the array of the string to specify the namespace.

```rust
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "document"])]
    fn write(s: &str);
}

write("hello, document!");
```

This example shows how to bind `window.document.write` in Rust.

If all items in the `extern "C" { … }` block have the same `js_namespace = …`:

```rust
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = Math)]
    fn random() -> f64;
    #[wasm_bindgen(js_namespace = Math)]
    fn log(a: f64) -> f64;
    // ...
}
```

Then that macro argument can also be moved to the outer block:

```rust
#[wasm_bindgen(js_namespace = Math)]
extern "C" {
    #[wasm_bindgen]
    fn random() -> f64;
    #[wasm_bindgen]
    fn log(a: f64) -> f64;
    // ...
}
```

`js_namespace = …` on an individual item takes precedence over the outer block's `js_namespace = …`.
