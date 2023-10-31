# cc-rs

A library to compile C/C++/assembly into a Rust library/application.

[Documentation](https://docs.rs/cc)

A simple library meant to be used as a build dependency with Cargo packages in
order to build a set of C/C++ files into a static archive. This crate calls out
to the most relevant compiler for a platform, for example using `cl` on MSVC.

## Using cc-rs

First, you'll want to both add a build script for your crate (`build.rs`) and
also add this crate to your `Cargo.toml` via:

```toml
[build-dependencies]
cc = "1.0"
```

Next up, you'll want to write a build script like so:

```rust,no_run
// build.rs

fn main() {
    cc::Build::new()
        .file("foo.c")
        .file("bar.c")
        .compile("foo");
}
```

And that's it! Running `cargo build` should take care of the rest and your Rust
application will now have the C files `foo.c` and `bar.c` compiled into a file
named `libfoo.a`. If the C files contain

```c
void foo_function(void) { ... }
```

and

```c
int32_t bar_function(int32_t x) { ... }
```

you can call them from Rust by declaring them in
your Rust code like so:

```rust,no_run
extern {
    fn foo_function();
    fn bar_function(x: i32) -> i32;
}

pub fn call() {
    unsafe {
        foo_function();
        bar_function(42);
    }
}

fn main() {
    // ...
}
```

See [the Rustonomicon](https://doc.rust-lang.org/nomicon/ffi.html) for more details.

## External configuration via environment variables

To control the programs and flags used for building, the builder can set a
number of different environment variables.

* `CFLAGS` - a series of space separated flags passed to compilers. Note that
             individual flags cannot currently contain spaces, so doing
             something like: `-L=foo\ bar` is not possible.
* `CC` - the actual C compiler used. Note that this is used as an exact
         executable name, so (for example) no extra flags can be passed inside
         this variable, and the builder must ensure that there aren't any
         trailing spaces. This compiler must understand the `-c` flag. For
         certain `TARGET`s, it also is assumed to know about other flags (most
         common is `-fPIC`).
* `AR` - the `ar` (archiver) executable to use to build the static library.
* `CRATE_CC_NO_DEFAULTS` - the default compiler flags may cause conflicts in some cross compiling scenarios. Setting this variable will disable the generation of default compiler flags.
* `CXX...` - see [C++ Support](#c-support).

Each of these variables can also be supplied with certain prefixes and suffixes,
in the following prioritized order:

1. `<var>_<target>` - for example, `CC_x86_64-unknown-linux-gnu`
2. `<var>_<target_with_underscores>` - for example, `CC_x86_64_unknown_linux_gnu`
3. `<build-kind>_<var>` - for example, `HOST_CC` or `TARGET_CFLAGS`
4. `<var>` - a plain `CC`, `AR` as above.

If none of these variables exist, cc-rs uses built-in defaults

In addition to the above optional environment variables, `cc-rs` has some
functions with hard requirements on some variables supplied by [cargo's
build-script driver][cargo] that it has the `TARGET`, `OUT_DIR`, `OPT_LEVEL`,
and `HOST` variables.

[cargo]: https://doc.rust-lang.org/cargo/reference/build-scripts.html#inputs-to-the-build-script

## Optional features

### Parallel

Currently cc-rs supports parallel compilation (think `make -jN`) but this
feature is turned off by default. To enable cc-rs to compile C/C++ in parallel,
you can change your dependency to:

```toml
[build-dependencies]
cc = { version = "1.0", features = ["parallel"] }
```

By default cc-rs will limit parallelism to `$NUM_JOBS`, or if not present it
will limit it to the number of cpus on the machine. If you are using cargo,
use `-jN` option of `build`, `test` and `run` commands as `$NUM_JOBS`
is supplied by cargo.

## Compile-time Requirements

To work properly this crate needs access to a C compiler when the build script
is being run. This crate does not ship a C compiler with it. The compiler
required varies per platform, but there are three broad categories:

* Unix platforms require `cc` to be the C compiler. This can be found by
  installing cc/clang on Linux distributions and Xcode on macOS, for example.
* Windows platforms targeting MSVC (e.g. your target triple ends in `-msvc`)
  require `cl.exe` to be available and in `PATH`. This is typically found in
  standard Visual Studio installations and the `PATH` can be set up by running
  the appropriate developer tools shell.
* Windows platforms targeting MinGW (e.g. your target triple ends in `-gnu`)
  require `cc` to be available in `PATH`. We recommend the
  [MinGW-w64](https://www.mingw-w64.org/) distribution, which is using the
  [Win-builds](http://win-builds.org/) installation system.
  You may also acquire it via
  [MSYS2](https://www.msys2.org/), as explained [here][msys2-help].  Make sure
  to install the appropriate architecture corresponding to your installation of
  rustc. GCC from older [MinGW](http://www.mingw.org/) project is compatible
  only with 32-bit rust compiler.

[msys2-help]: https://github.com/rust-lang/rust#building-on-windows

## C++ support

`cc-rs` supports C++ libraries compilation by using the `cpp` method on
`Build`:

```rust,no_run
fn main() {
    cc::Build::new()
        .cpp(true) // Switch to C++ library compilation.
        .file("foo.cpp")
        .compile("libfoo.a");
}
```

For C++ libraries, the `CXX` and `CXXFLAGS` environment variables are used instead of `CC` and `CFLAGS`.

The C++ standard library may be linked to the crate target. By default it's `libc++` for macOS, FreeBSD, and OpenBSD, `libc++_shared` for Android, nothing for MSVC, and `libstdc++` for anything else. It can be changed in one of two ways:

1. by using the `cpp_link_stdlib` method on `Build`:
    ```rust,no-run
    fn main() {
        cc::Build::new()
            .cpp(true)
            .file("foo.cpp")
            .cpp_link_stdlib("stdc++") // use libstdc++
            .compile("libfoo.a");
    }
    ```
2. by setting the `CXXSTDLIB` environment variable.

In particular, for Android you may want to [use `c++_static` if you have at most one shared library](https://developer.android.com/ndk/guides/cpp-support).

Remember that C++ does name mangling so `extern "C"` might be required to enable Rust linker to find your functions.

## CUDA C++ support

`cc-rs` also supports compiling CUDA C++ libraries by using the `cuda` method
on `Build` (currently for GNU/Clang toolchains only):

```rust,no_run
fn main() {
    cc::Build::new()
        // Switch to CUDA C++ library compilation using NVCC.
        .cuda(true)
        .cudart("static")
        // Generate code for Maxwell (GTX 970, 980, 980 Ti, Titan X).
        .flag("-gencode").flag("arch=compute_52,code=sm_52")
        // Generate code for Maxwell (Jetson TX1).
        .flag("-gencode").flag("arch=compute_53,code=sm_53")
        // Generate code for Pascal (GTX 1070, 1080, 1080 Ti, Titan Xp).
        .flag("-gencode").flag("arch=compute_61,code=sm_61")
        // Generate code for Pascal (Tesla P100).
        .flag("-gencode").flag("arch=compute_60,code=sm_60")
        // Generate code for Pascal (Jetson TX2).
        .flag("-gencode").flag("arch=compute_62,code=sm_62")
        .file("bar.cu")
        .compile("libbar.a");
}
```

## License

This project is licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or
   https://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or
   https://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in cc-rs by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
