# JS Snippets

Often when developing a crate you want to run on the web you'll want to include
some JS code here and there. While [`js-sys`](https://docs.rs/js-sys) and
[`web-sys`](https://docs.rs/web-sys) cover many needs they don't cover
everything, so `wasm-bindgen` supports the ability to write JS code next to your
Rust code and have it included in the final output artifact.

To include a local JS file, you'll use the `#[wasm_bindgen(module)]` macro:

```rust
#[wasm_bindgen(module = "/js/foo.js")]
extern "C" {
    fn add(a: u32, b: u32) -> u32;
}
```

This declaration indicates that all the functions contained in the `extern`
block are imported from the file `/js/foo.js`, where the root is relative to the
crate root (where `Cargo.toml` is located).

The `/js/foo.js` file will make its way to the final output when `wasm-bindgen`
executes, so you can use the `module` annotation in a library without having to
worry users of your library!

The JS file itself must be written with ES module syntax:

```js
export function add(a, b) {
    return a + b;
}
```

A full design of this feature can be found in [RFC 6] as well if you're
interested!

[RFC 6]: https://github.com/rustwasm/rfcs/pull/6

### Using `inline_js`

In addition to `module = "..."` if you're a macro author you also have the
ability to use the `inline_js` attribute:

```rust
#[wasm_bindgen(inline_js = "export function add(a, b) { return a + b; }")]
extern "C" {
    fn add(a: u32, b: u32) -> u32;
}
```

Using `inline_js` indicates that the JS module is specified inline in the
attribute itself, and no files are loaded from the filesystem. They have the
same limitations and caveats as when using `module`, but can sometimes be easier
to generate for macros themselves. It's not recommended for hand-written code to
make use of `inline_js` but instead to leverage `module` where possible.

### Caveats

While quite useful local JS snippets currently suffer from a few caveats which
are important to be aware of. Many of these are temporary though!

* Currently `import` statements are not supported in the JS file. This is a
  restriction we may lift in the future once we settle on a good way to support
  this. For now, though, js snippets must be standalone modules and can't import
  from anything else.

* Only `--target web` and the default bundler output mode are supported. To
  support `--target nodejs` we'd need to translate ES module syntax to CommonJS
  (this is
  planned to be done, just hasn't been done yet). Additionally to support
  `--target no-modules` we'd have to similarly translate from ES modules to
  something else.

* Paths in `module = "..."` must currently start with `/`, or be rooted at the
  crate root. It is intended to eventually support relative paths like `./` and
  `../`, but it's currently believed that this requires more support in
  the Rust `proc_macro` crate.

As above, more detail about caveats can be found in [RFC 6].
