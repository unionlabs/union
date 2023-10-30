# toml-rs

[![Latest Version](https://img.shields.io/crates/v/toml.svg)](https://crates.io/crates/toml)
[![Documentation](https://docs.rs/toml/badge.svg)](https://docs.rs/toml)

A [TOML][toml] decoder and encoder for Rust. This library is currently compliant
with the v0.5.0 version of TOML. This library will also likely continue to stay
up to date with the TOML specification as changes happen.

[toml]: https://github.com/toml-lang/toml

```toml
# Cargo.toml
[dependencies]
toml = "0.5"
```

This crate also supports serialization/deserialization through the
[serde](https://serde.rs) crate on crates.io. Currently the older `rustc-serialize`
crate is not supported in the 0.3+ series of the `toml` crate, but 0.2 can be
used for that support.

# License

This project is licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or
   http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or
   http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in toml-rs by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
