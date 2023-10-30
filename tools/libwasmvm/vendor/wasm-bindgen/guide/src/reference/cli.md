# The `wasm-bindgen` Command Line Interface

The `wasm-bindgen` command line tool has a number of options available to it to
tweak the JavaScript that is generated. The most up-to-date set of flags can
always be listed via `wasm-bindgen --help`.

## Installation

```
cargo install -f wasm-bindgen-cli
```

## Usage

```
wasm-bindgen [options] ./target/wasm32-unknown-unknown/release/crate.wasm
```

## Options

### `--out-dir DIR`

The target directory to emit the JavaScript bindings, TypeScript definitions,
processed `.wasm` binary, etc...

### `--target`

This flag indicates what flavor of output what `wasm-bindgen` should generate.
For example it could generate code to be loaded in a bundler like Webpack, a
native web page, or Node.js. For a full list of options to pass this flag, see
the section on [deployment]

[deployment]: deployment.html

### `--no-modules-global VAR`

When `--target no-modules` is used this flag can indicate what the name of the
global to assign generated bindings to.

For more information about this see the section on [deployment]

### `--typescript`

Output a TypeScript declaration file for the generated JavaScript bindings. This
is on by default.

### `--no-typescript`

By default, a `*.d.ts` TypeScript declaration file is generated for the
generated JavaScript bindings, but this flag will disable that.

### `--omit-imports`

When the `module` attribute is used with the `wasm-bindgen` macro, the code
generator will emit corresponding `import` or `require` statements in the header
section of the generated javascript. This flag causes those import statements to
be omitted. This is necessary for some use cases, such as generating javascript
which is intended to be used with Electron (with node integration disabled),
where the imports are instead handled through a separate preload script.

### `--debug`

Generates a bit more JS and wasm in "debug mode" to help catch programmer
errors, but this output isn't intended to be shipped to production.

### `--no-demangle`

When post-processing the `.wasm` binary, do not demangle Rust symbols in the
"names" custom section.

### `--keep-lld-exports`

When post-processing the `.wasm` binary, do not remove exports that are
synthesized by Rust's linker, LLD.

### `--keep-debug`

When post-processing the `.wasm` binary, do not strip DWARF debug info custom
sections.

### `--browser`

When generating bundler-compatible code (see the section on [deployment]) this
indicates that the bundled code is always intended to go into a browser so a few
checks for Node.js can be elided.

### `--weak-refs`

Enables usage of the [TC39 Weak References
proposal](https://github.com/tc39/proposal-weakrefs), ensuring that all Rust
memory is eventually deallocated regardless of whether you're calling `free` or
not. This is off-by-default while we're waiting for support to percolate into
all major browsers. For more information see the [documentation about weak
references](./weak-references.md).

### `--reference-types`

Enables usage of the [WebAssembly References Types
proposal](https://github.com/webassembly/reference-types) proposal, meaning that
the WebAssembly binary will use `externref` when importing and exporting
functions that work with `JsValue`. For more information see the [documentation
about reference types](./reference-types.md).

### `--omit-default-module-path`

Don't add WebAssembly fallback imports in generated JavaScript.

### `--split-linked-modules`

Controls whether wasm-bindgen will split linked modules out into their own
files. Enabling this is recommended, because it allows lazy-loading the linked
modules and setting a stricter Content Security Policy.

wasm-bindgen uses the `new URL('…', import.meta.url)` syntax to resolve the
links to such split out files. This breaks with most bundlers, since the bundler
doesn't know to include the linked module in its output. That's why this option
is disabled by default. Webpack 5 is an exception, which has special treatment
for that syntax.

For other bundlers, you'll need to take extra steps to get it to work, likely by
using a plugin. Alternatively, you can leave the syntax as is and instead
manually configure the bundler to copy all files in `snippets/` to the output
directory, preserving their paths relative to whichever bundled file ends up
containing the JS shim.

On the no-modules target, `link_to!` won't work if used outside of a document,
e.g. inside a worker. This is because it's impossible to figure out what the
URL of the linked module is without a reference point like `import.meta.url`.
