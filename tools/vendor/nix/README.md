# Rust bindings to *nix APIs

[![Cirrus Build Status](https://api.cirrus-ci.com/github/nix-rust/nix.svg)](https://cirrus-ci.com/github/nix-rust/nix)
[![crates.io](https://img.shields.io/crates/v/nix.svg)](https://crates.io/crates/nix)

[Documentation (Releases)](https://docs.rs/nix/)

Nix seeks to provide friendly bindings to various *nix platform APIs (Linux, Darwin,
...). The goal is to not provide a 100% unified interface, but to unify
what can be while still providing platform specific APIs.

For many system APIs, Nix provides a safe alternative to the unsafe APIs
exposed by the [libc crate](https://github.com/rust-lang/libc).  This is done by
wrapping the libc functionality with types/abstractions that enforce legal/safe
usage.


As an example of what Nix provides, examine the differences between what is
exposed by libc and nix for the
[gethostname](https://man7.org/linux/man-pages/man2/gethostname.2.html) system
call:

```rust,ignore
// libc api (unsafe, requires handling return code/errno)
pub unsafe extern fn gethostname(name: *mut c_char, len: size_t) -> c_int;

// nix api (returns a nix::Result<CStr>)
pub fn gethostname<'a>(buffer: &'a mut [u8]) -> Result<&'a CStr>;
```

## Supported Platforms

nix target support consists of two tiers. While nix attempts to support all
platforms supported by [libc](https://github.com/rust-lang/libc), only some
platforms are actively supported due to either technical or manpower
limitations. Support for platforms is split into three tiers:

  * Tier 1 - Builds and tests for this target are run in CI. Failures of either
             block the inclusion of new code.
  * Tier 2 - Builds for this target are run in CI. Failures during the build
             blocks the inclusion of new code. Tests may be run, but failures
             in tests don't block the inclusion of new code.
  * Tier 3 - Builds for this target are run in CI. Failures during the build
             *do not* block the inclusion of new code. Testing may be run, but
             failures in tests don't block the inclusion of new code.

The following targets are supported by `nix`:

Tier 1:
  * aarch64-unknown-linux-gnu
  * arm-unknown-linux-gnueabi
  * armv7-unknown-linux-gnueabihf
  * i686-unknown-freebsd
  * i686-unknown-linux-gnu
  * i686-unknown-linux-musl
  * mips-unknown-linux-gnu
  * mips64-unknown-linux-gnuabi64
  * mips64el-unknown-linux-gnuabi64
  * mipsel-unknown-linux-gnu
  * powerpc64le-unknown-linux-gnu
  * x86_64-apple-darwin
  * x86_64-unknown-freebsd
  * x86_64-unknown-linux-gnu
  * x86_64-unknown-linux-musl

Tier 2:
  * aarch64-apple-ios
  * aarch64-linux-android
  * arm-linux-androideabi
  * arm-unknown-linux-musleabi
  * armv7-linux-androideabi
  * i686-linux-android
  * powerpc-unknown-linux-gnu
  * s390x-unknown-linux-gnu
  * x86_64-apple-ios
  * x86_64-linux-android
  * x86_64-unknown-illumos
  * x86_64-unknown-netbsd

Tier 3:
  * x86_64-fuchsia
  * x86_64-unknown-dragonfly
  * x86_64-unknown-linux-gnux32
  * x86_64-unknown-openbsd
  * x86_64-unknown-redox

## Minimum Supported Rust Version (MSRV)

nix is supported on Rust 1.46.0 and higher.  It's MSRV will not be
changed in the future without bumping the major or minor version.

## Contributing

Contributions are very welcome.  Please See [CONTRIBUTING](CONTRIBUTING.md) for
additional details.

Feel free to join us in [the nix-rust/nix](https://gitter.im/nix-rust/nix) channel on Gitter to
discuss `nix` development.

## License

Nix is licensed under the MIT license.  See [LICENSE](LICENSE) for more details.
