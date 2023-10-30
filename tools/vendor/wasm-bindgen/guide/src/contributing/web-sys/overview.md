# `web-sys` Overview

The `web-sys` crate has this file and directory layout:

```text
.
├── build.rs
├── Cargo.toml
├── README.md
├── src
│   └── lib.rs
└── webidls
    └── enabled
        └── ...
```

### `webidls/enabled/*.webidl`

These are the WebIDL interfaces that we will actually generate bindings for (or
at least bindings for *some* of the things defined in these files).

### `build.rs`

The `build.rs` invokes `wasm-bindgen`'s WebIDL frontend on all the WebIDL files
in `webidls/enabled`. It writes the resulting bindings into the cargo build's
out directory.

### `src/lib.rs`

The only thing `src/lib.rs` does is include the bindings generated at compile
time in `build.rs`. Here is the whole `src/lib.rs` file:

```rust
{{#include ../../../../crates/web-sys/src/lib.rs}}
```

### Cargo features

When compiled the crate is almost empty by default, which probably isn't what
you want! Due to the very large number of APIs, this crate uses features to
enable portions of its API to reduce compile times. The list of features in
`Cargo.toml` all correspond to types in the generated functions. Enabling a
feature enables that type. All methods should indicate what features need to be
activated to use the method.
