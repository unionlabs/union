# [RustCrypto]: CPU Feature Detection

[![crate][crate-image]][crate-link]
[![Docs][docs-image]][docs-link]
![Apache2/MIT licensed][license-image]
![Rust Version][rustc-image]
[![Project Chat][chat-image]][chat-link]
[![Build Status][build-image]][build-link]

Lightweight and efficient runtime CPU feature detection for `aarch64` and
`x86`/`x86_64` targets.

Supports `no_std` as well as mobile targets including iOS and Android,
providing an alternative to the `std`-dependent `is_x86_feature_detected!`
macro.

[Documentation][docs-link]

## Supported architectures

### `aarch64`: Android, iOS, Linux, and macOS/M4 only

Note: ARM64 does not support OS-independent feature detection, so support must
be implemented on an OS-by-OS basis.

Target features:

- `aes`
- `sha2`
- `sha3`

Note: please open a GitHub Issue to request support for additional features.

### `x86`/`x86_64`: OS independent and `no_std`-friendly

Target features:

- `adx`
- `aes`
- `avx`
- `avx2`
- `bmi1`
- `bmi2`
- `fma`
- `mmx`
- `pclmulqdq`
- `popcnt`
- `rdrand`
- `rdseed`
- `sgx`
- `sha`
- `sse`
- `sse2`
- `sse3`
- `sse4.1`
- `sse4.2`
- `ssse3`

## License

Licensed under either of:

 * [Apache License, Version 2.0](http://www.apache.org/licenses/LICENSE-2.0)
 * [MIT license](http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.

[//]: # (badges)

[crate-image]: https://img.shields.io/crates/v/cpufeatures.svg
[crate-link]: https://crates.io/crates/cpufeatures
[docs-image]: https://docs.rs/cpufeatures/badge.svg
[docs-link]: https://docs.rs/cpufeatures/
[license-image]: https://img.shields.io/badge/license-Apache2.0/MIT-blue.svg
[rustc-image]: https://img.shields.io/badge/rustc-1.40+-blue.svg
[chat-image]: https://img.shields.io/badge/zulip-join_chat-blue.svg
[chat-link]: https://rustcrypto.zulipchat.com/#narrow/stream/260052-utils
[build-image]: https://github.com/RustCrypto/utils/workflows/cpufeatures/badge.svg?branch=master&event=push
[build-link]: https://github.com/RustCrypto/utils/actions/workflows/cpufeatures.yml

[//]: # (general links)

[RustCrypto]: https://github.com/rustcrypto
[RustCrypto/utils#378]: https://github.com/RustCrypto/utils/issues/378
