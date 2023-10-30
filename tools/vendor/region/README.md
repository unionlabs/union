<div align="center">

# `region-rs`

## Cross-platform virtual memory API

[![GitHub CI Status][github-shield]][github]
[![crates.io version][crate-shield]][crate]
[![Documentation][docs-shield]][docs]
[![License][license-shield]][license]

 </div>

This crate provides a cross-platform Rust API for allocating, querying and
manipulating virtual memory. It is a thin abstraction, with the underlying
interaction implemented using platform specific APIs (e.g `VirtualQuery`,
`VirtualAlloc`, `VirtualLock`, `mprotect`, `mmap`, `mlock`).

## Platforms

This library is continuously tested against these targets:

- Linux
  * `aarch64-linux-android`
  * `armv7-unknown-linux-gnueabihf`
  * `i686-unknown-linux-gnu`
  * `mips-unknown-linux-gnu`
  * `x86_64-unknown-linux-gnu`
  * `x86_64-unknown-linux-musl`
- Windows
  * `i686-pc-windows-gnu`
  * `i686-pc-windows-msvc`
  * `x86_64-pc-windows-gnu`
  * `x86_64-pc-windows-msvc`
- macOS
  * `x86_64-apple-darwin`
- FreeBSD
  * `x86_64-unknown-freebsd`
- OpenBSD
  * `x86_64-unknown-openbsd`

... and continuously checked against these targets:

- Illumos
  * `x86_64-unknown-illumos`

Beyond the aformentioned target triplets, the library is also expected to work
against a multitude of omitted architectures.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
region = "3.0.0"
```

## Example

- Cross-platform equivalents:
```rust
let data = [0xDE, 0xAD, 0xBE, 0xEF];

// Page size
let pz = region::page::size();

// VirtualQuery | '/proc/self/maps'
let q  = region::query(data.as_ptr())?;
let qr = region::query_range(data.as_ptr(), data.len())?;

// VirtualAlloc | mmap
let alloc = region::alloc(100, Protection::READ_WRITE)?;

// VirtualProtect | mprotect
region::protect(data.as_ptr(), data.len(), Protection::READ_WRITE_EXECUTE)?;

// ... you can also temporarily change one or more pages' protection
let handle = region::protect_with_handle(data.as_ptr(), data.len(), Protection::READ_WRITE_EXECUTE)?;

// VirtualLock | mlock
let guard = region::lock(data.as_ptr(), data.len())?;
```

<!-- Links -->
[github-shield]: https://img.shields.io/github/workflow/status/darfink/region-rs/CI/master?label=actions&logo=github&style=for-the-badge
[github]: https://github.com/darfink/region-rs/actions/workflows/ci.yml?query=branch%3Amaster
[crate-shield]: https://img.shields.io/crates/v/region.svg?style=for-the-badge
[crate]: https://crates.io/crates/region
[docs-shield]: https://img.shields.io/badge/docs-crates-green.svg?style=for-the-badge
[docs]: https://docs.rs/region/
[license-shield]: https://img.shields.io/crates/l/region.svg?style=for-the-badge
[license]: https://github.com/darfink/region-rs