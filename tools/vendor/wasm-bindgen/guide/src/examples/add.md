# Small wasm files

[View full source code][code] or [view the compiled example online][online]

[online]: https://rustwasm.github.io/wasm-bindgen/exbuild/add/
[code]: https://github.com/rustwasm/wasm-bindgen/tree/master/examples/add

One of `wasm-bindgen`'s core goals is a pay-only-for-what-you-use philosophy, so
if we don't use much then we shouldn't be paying much! As a result
`#[wasm_bindgen]` can generate super-small executables

Currently this code...

```rust
{{#include ../../../examples/add/src/lib.rs}}
```

generates a 710 byte wasm binary:

```
$ ls -l add_bg.wasm
-rw-rw-r-- 1 alex alex 710 Sep 19 17:32 add_bg.wasm
```

If you run [wasm-opt], a C++ tool for optimize WebAssembly, you can make it
even smaller too!

```
$ wasm-opt -Os add_bg.wasm -o add.wasm
$ ls -l add.wasm
-rw-rw-r-- 1 alex alex 172 Sep 19 17:33 add.wasm
```

And sure enough, using the [wasm2wat] tool it's quite small!

```
$ wasm2wat add.wasm
(module
  (type (;0;) (func (param i32 i32) (result i32)))
  (func (;0;) (type 0) (param i32 i32) (result i32)
    get_local 1
    get_local 0
    i32.add)
  (table (;0;) 1 1 anyfunc)
  (memory (;0;) 17)
  (global (;0;) i32 (i32.const 1049118))
  (global (;1;) i32 (i32.const 1049118))
  (export "memory" (memory 0))
  (export "__indirect_function_table" (table 0))
  (export "__heap_base" (global 0))
  (export "__data_end" (global 1))
  (export "add" (func 0))
  (data (i32.const 1049096) "invalid malloc request"))
```

Also don't forget to compile in release mode for the smallest binaries! For
larger applications you'll likely also want to turn on LTO to generate the
smallest binaries:

```toml
[profile.release]
lto = true
```

[wasm2wat]: https://github.com/webassembly/wabt
[wasm-opt]: https://github.com/webassembly/binaryen
