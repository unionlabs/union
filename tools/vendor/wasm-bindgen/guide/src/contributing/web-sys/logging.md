# Logging

The `wasm_bindgen_webidl` crate (used by `web-sys`'s `build.rs`) uses
[`env_logger`][env_logger] for logging, which can be enabled by setting the
`RUST_LOG=wasm_bindgen_webidl` environment variable while building the `web-sys`
crate.

Make sure to enable "very verbose" output during `cargo build` to see these logs
within `web-sys`'s build script output.

```sh
cd crates/web-sys
RUST_LOG=wasm_bindgen_webidl cargo build -vv
```

If `wasm_bindgen_webidl` encounters WebIDL constructs that it doesn't know how
to translate into `wasm-bindgen` AST items, it will emit warn-level logs.

```
WARN 2018-07-06T18:21:49Z: wasm_bindgen_webidl: Unsupported WebIDL interface: ...
```

[env_logger]: https://crates.io/crates/env_logger
