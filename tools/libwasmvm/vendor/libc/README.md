# libc - Raw FFI bindings to platforms' system libraries

[![GHA Status]][GitHub Actions] [![Cirrus CI Status]][Cirrus CI] [![Latest Version]][crates.io] [![Documentation]][docs.rs] ![License]

`libc` provides all of the definitions necessary to easily interoperate with C
code (or "C-like" code) on each of the platforms that Rust supports. This
includes type definitions (e.g. `c_int`), constants (e.g. `EINVAL`) as well as
function headers (e.g. `malloc`).

This crate exports all underlying platform types, functions, and constants under
the crate root, so all items are accessible as `libc::foo`. The types and values
of all the exported APIs match the platform that libc is compiled for.

More detailed information about the design of this library can be found in its
[associated RFC][rfc].

[rfc]: https://github.com/rust-lang/rfcs/blob/HEAD/text/1291-promote-libc.md

## Usage

Add the following to your `Cargo.toml`:

```toml
[dependencies]
libc = "0.2"
```

## Features

* `std`: by default `libc` links to the standard library. Disable this
  feature to remove this dependency and be able to use `libc` in `#![no_std]`
  crates.

* `extra_traits`: all `struct`s implemented in `libc` are `Copy` and `Clone`.
  This feature derives `Debug`, `Eq`, `Hash`, and `PartialEq`.

* `const-extern-fn`: Changes some `extern fn`s into `const extern fn`s.
  If you use Rust >= 1.62, this feature is implicitly enabled.
  Otherwise it requires a nightly rustc.

* **deprecated**: `use_std` is deprecated, and is equivalent to `std`.

## Rust version support

The minimum supported Rust toolchain version is currently **Rust 1.13.0**.
(libc does not currently have any policy regarding changes to the minimum
supported Rust version; such policy is a work in progress.) APIs requiring
newer Rust features are only available on newer Rust toolchains:

| Feature              | Version |
|----------------------|---------|
| `union`              |  1.19.0 |
| `const mem::size_of` |  1.24.0 |
| `repr(align)`        |  1.25.0 |
| `extra_traits`       |  1.25.0 |
| `core::ffi::c_void`  |  1.30.0 |
| `repr(packed(N))`    |  1.33.0 |
| `cfg(target_vendor)` |  1.33.0 |
| `const-extern-fn`    |  1.62.0 |

## Platform support

[Platform-specific documentation (HEAD)][docs.head].

See
[`ci/build.sh`](https://github.com/rust-lang/libc/blob/HEAD/ci/build.sh)
for the platforms on which `libc` is guaranteed to build for each Rust
toolchain. The test-matrix at [GitHub Actions] and [Cirrus CI] show the
platforms in which `libc` tests are run.

<div class="platform_docs"></div>

## License

This project is licensed under either of

* [Apache License, Version 2.0](https://www.apache.org/licenses/LICENSE-2.0)
  ([LICENSE-APACHE](https://github.com/rust-lang/libc/blob/HEAD/LICENSE-APACHE))

* [MIT License](https://opensource.org/licenses/MIT)
  ([LICENSE-MIT](https://github.com/rust-lang/libc/blob/HEAD/LICENSE-MIT))

at your option.

## Contributing

We welcome all people who want to contribute. Please see the [contributing
instructions] for more information.

[contributing instructions]: https://github.com/rust-lang/libc/blob/HEAD/CONTRIBUTING.md

Contributions in any form (issues, pull requests, etc.) to this project
must adhere to Rust's [Code of Conduct].

[Code of Conduct]: https://www.rust-lang.org/policies/code-of-conduct

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in `libc` by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.

[GitHub Actions]: https://github.com/rust-lang/libc/actions
[GHA Status]: https://github.com/rust-lang/libc/workflows/CI/badge.svg
[Cirrus CI]: https://cirrus-ci.com/github/rust-lang/libc
[Cirrus CI Status]: https://api.cirrus-ci.com/github/rust-lang/libc.svg
[crates.io]: https://crates.io/crates/libc
[Latest Version]: https://img.shields.io/crates/v/libc.svg
[Documentation]: https://docs.rs/libc/badge.svg
[docs.rs]: https://docs.rs/libc
[License]: https://img.shields.io/crates/l/libc.svg
[docs.head]: https://rust-lang.github.io/libc/#platform-specific-documentation
