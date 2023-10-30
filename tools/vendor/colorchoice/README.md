# colorchoice

> Global override of color control

[![Documentation](https://img.shields.io/badge/docs-master-blue.svg)][Documentation]
![License](https://img.shields.io/crates/l/colorchoice.svg)
[![Crates Status](https://img.shields.io/crates/v/colorchoice.svg)](https://crates.io/crates/colorchoice)

## [Contribute](../../CONTRIBUTING.md)

Special note: to be successful, this crate **cannot** break compatibility or
else different crates in the hierarchy will be reading different globals.
While end users can work around this, it isn't ideal.    If we need a new API, we can make
the old API an adapter to the new logic.

Similarly, we should strive to reduce **risk** of breaking compatibility by
exposing as little as possible.  Anything more should be broken out into a
separate crate that this crate can call into.

## License

Dual-licensed under [MIT](../../LICENSE-MIT) or [Apache 2.0](../../LICENSE-APACHE)

[Documentation]: https://docs.rs/colorchoice
