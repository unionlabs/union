# `raw_module = "blah"`

This attribute performs exactly the same purpose as the [`module`
attribute](module.html) on JS imports, but it does not attempt to interpret
paths starting with `./`, `../`, or `/` as JS snippets. For example:

```rust
#[wasm_bindgen(raw_module = "./some/js/file.js")]
extern "C" {
    fn the_function();
}
```

Note that if you use this attribute with a relative or absolute path, it's
likely up to the final bundler or project to assign meaning to that path. This
typically means that the JS file or module will be resolved relative to the
final location of the wasm file itself. That means that `raw_module` is likely
unsuitable for libraries on crates.io, but may be usable within end-user
applications.
