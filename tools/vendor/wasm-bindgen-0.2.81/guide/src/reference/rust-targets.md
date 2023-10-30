# Supported Rust Targets

> **Note**: This section is about Rust target triples, not targets like node/web
> workers/browsers. More information on that coming soon!

The `wasm-bindgen` project is designed to target the `wasm32-unknown-unknown`
target in Rust. This target is a "bare bones" target for Rust which emits
WebAssembly as output. The standard library is largely inert as modules like
`std::fs` and `std::net` will simply return errors.

## Non-wasm targets

Note that `wasm-bindgen` also aims to compile on all targets. This means that it
should be safe, if you like, to use `#[wasm_bindgen]` even when compiling for
Windows (for example). For example:

```rust
#[wasm_bindgen]
pub fn add(a: u32, b: u32) -> u32 {
    a + b
}

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    println!("1 + 2 = {}", add(1, 2));
}
```

This program will compile and work on all platforms, not just
`wasm32-unknown-unknown`. Note that imported functions with `#[wasm_bindgen]`
will unconditionally panic on non-wasm targets. For example:

```rust
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

fn main() {
    log("hello!");
}
```

This program will unconditionally panic on all platforms other than
`wasm32-unknown-unknown`.

For better compile times, however, you likely want to only use `#[wasm_bindgen]`
on the `wasm32-unknown-unknown` target. You can have a target-specific
dependency like so:

```toml
[target.'cfg(target_arch = "wasm32")'.dependencies]
wasm-bindgen = "0.2"
```

And in your code you can use:

```rust
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn only_on_the_wasm_target() {
    // ...
}
```

## Other Web Targets

The `wasm-bindgen` target does not support the `wasm32-unknown-emscripten` nor
the `asmjs-unknown-emscripten` targets. There are currently no plans to support
these targets either. All annotations work like other platforms on the targets,
retaining exported functions and causing all imports to panic.
