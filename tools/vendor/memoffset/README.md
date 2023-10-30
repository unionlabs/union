# memoffset #

[![](https://img.shields.io/crates/v/memoffset.svg)](https://crates.io/crates/memoffset)

C-Like `offset_of` functionality for Rust structs.

Introduces the following macros:
 * `offset_of!` for obtaining the offset of a member of a struct.
 * `offset_of_tuple!` for obtaining the offset of a member of a tuple. (Requires Rust 1.20+)
 * `offset_of_union!` for obtaining the offset of a member of a union.
 * `span_of!` for obtaining the range that a field, or fields, span.

`memoffset` works under `no_std` environments.

## Usage ##
Add the following dependency to your `Cargo.toml`:

```toml
[dependencies]
memoffset = "0.8"
```

These versions will compile fine with rustc versions greater or equal to 1.19.

## Examples ##
```rust
use memoffset::{offset_of, span_of};

#[repr(C, packed)]
struct Foo {
    a: u32,
    b: u32,
    c: [u8; 5],
    d: u32,
}

fn main() {
    assert_eq!(offset_of!(Foo, b), 4);
    assert_eq!(offset_of!(Foo, d), 4+4+5);

    assert_eq!(span_of!(Foo, a),        0..4);
    assert_eq!(span_of!(Foo, a ..  c),  0..8);
    assert_eq!(span_of!(Foo, a ..= c),  0..13);
    assert_eq!(span_of!(Foo, ..= d),    0..17);
    assert_eq!(span_of!(Foo, b ..),     4..17);
}
```

## Usage in constants ##
`memoffset` has support for compile-time `offset_of!` on rust>=1.65, or on older nightly compilers.

### Usage on stable Rust ###
Constant evaluation is automatically enabled and avilable on stable compilers starting with rustc 1.65.

This is an incomplete implementation with one caveat:
Due to dependence on [`#![feature(const_refs_to_cell)]`](https://github.com/rust-lang/rust/issues/80384), you cannot get the offset of a `Cell` field in a const-context.

This means that if need to get the offset of a cell, you'll have to remain on nightly for now.

### Usage on recent nightlies ###

If you're using a new-enough nightly and you require the ability to get the offset of a `Cell`,
you'll have to enable the `unstable_const` cargo feature, as well as enabling `const_refs_to_cell` in your crate root.

Do note that `unstable_const` is an unstable feature that is set to be removed in a future version of `memoffset`.

Cargo.toml:
```toml
[dependencies.memoffset]
version = "0.8"
features = ["unstable_const"]
```

Your crate root: (`lib.rs`/`main.rs`)
```rust,ignore
#![feature(const_refs_to_cell)]
```

### Usage on older nightlies ###
In order to use it on an older nightly compiler, you must enable the `unstable_const` crate feature and several compiler features.

Your crate root: (`lib.rs`/`main.rs`)
```rust,ignore
#![feature(const_ptr_offset_from, const_refs_to_cell)]
```
