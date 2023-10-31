# Decimal &emsp; [![Build Status]][actions] [![Latest Version]][crates.io] [![Docs Badge]][docs]

[Build Status]: https://img.shields.io/endpoint.svg?url=https%3A%2F%2Factions-badge.atrox.dev%2Fpaupino%2Frust-decimal%2Fbadge&label=build&logo=none
[actions]: https://actions-badge.atrox.dev/paupino/rust-decimal/goto
[Latest Version]: https://img.shields.io/crates/v/rust-decimal.svg
[crates.io]: https://crates.io/crates/rust-decimal
[Docs Badge]: https://docs.rs/rust_decimal/badge.svg
[docs]: https://docs.rs/rust_decimal

A Decimal number implementation written in pure Rust suitable for financial calculations that require significant integral and fractional digits with no round-off errors.

The binary representation consists of a 96 bit integer number, a scaling factor used to specify the decimal fraction and a 1 bit sign. Because of this representation, trailing zeros are preserved and may be exposed when in string form. These can be truncated using the `normalize` or `round_dp` functions.

## Installing

Using [`cargo-edit`](https://crates.io/crates/cargo-edit):

```sh
$ cargo add rust_decimal
```

In addition, if you would like to use the optimized macro for convenient creation of decimals:

```sh
$ cargo add rust_decimal_macros
```

Alternatively, you can edit your `Cargo.toml` directly and run `cargo update`:

```toml
[dependencies]
rust_decimal = "1.28"
rust_decimal_macros = "1.28"
```

## Usage

Decimal numbers can be created in a few distinct ways. The easiest and most efficient method of creating a Decimal is to use the procedural macro within the `rust_decimal_macros` crate:

```rust
// Procedural macros need importing directly
use rust_decimal_macros::dec;

let number = dec!(-1.23) + dec!(3.45);
assert_eq!(number, dec!(2.22));
assert_eq!(number.to_string(), "2.22");
```

Alternatively you can also use one of the Decimal number convenience functions ([see the docs](https://docs.rs/rust_decimal/) for more details):

```rust
// Using the prelude can help importing trait based functions (e.g. core::str::FromStr).
use rust_decimal::prelude::*;

// Using an integer followed by the decimal points
let scaled = Decimal::new(202, 2);
assert_eq!("2.02", scaled.to_string());

// From a 128 bit integer
let balance = Decimal::from_i128_with_scale(5_897_932_384_626_433_832, 2);
assert_eq!("58979323846264338.32", balance.to_string());

// From a string representation
let from_string = Decimal::from_str("2.02").unwrap();
assert_eq!("2.02", from_string.to_string());

// From a string representation in a different base
let from_string_base16 = Decimal::from_str_radix("ffff", 16).unwrap();
assert_eq!("65535", from_string_base16.to_string());

// From scientific notation
let sci = Decimal::from_scientific("9.7e-7").unwrap();
assert_eq!("0.00000097", sci.to_string());

// Using the `Into` trait
let my_int: Decimal = 3_i32.into();
assert_eq!("3", my_int.to_string());

// Using the raw decimal representation
let pi = Decimal::from_parts(1_102_470_952, 185_874_565, 1_703_060_790, false, 28);
assert_eq!("3.1415926535897932384626433832", pi.to_string());
```

Once you have instantiated your `Decimal` number you can perform calculations with it just like any other number:

```rust
use rust_decimal::prelude::*;
use rust_decimal_macros::dec;

let amount = dec!(25.12);
let tax_percentage = dec!(0.085);
let total = amount + (amount * tax_percentage).round_dp(2);
assert_eq!(total, dec!(27.26));
```

## Features

**Behavior / Functionality**

* [borsh](#borsh)
* [c-repr](#c-repr)
* [legacy-ops](#legacy-ops)
* [maths](#maths)
* [rkyv](#rkyv)
* [rocket-traits](#rocket-traits)
* [rust-fuzz](#rust-fuzz)
* [std](#std)

**Database**

* [db-postgres](#db-postgres)
* [db-tokio-postgres](#db-tokio-postgres)
* [db-diesel-postgres](#db-diesel-postgres)
* [db-diesel-mysql](#db-diesel-mysql)

**Serde**

* [serde-float](#serde-float)
* [serde-str](#serde-str)
* [serde-arbitrary-precision](#serde-arbitrary-precision)
* [serde-with-float](#serde-with-float)
* [serde-with-str](#serde-with-str)
* [serde-with-arbitrary-precision](#serde-with-arbitrary-precision)

### `borsh`

Enables [Borsh](https://borsh.io/) serialization for `Decimal`.

### `c-repr`

Forces `Decimal` to use `[repr(C)]`. The corresponding target layout is 128 bit aligned.

### `db-postgres`

Enables a PostgreSQL communication module. It allows for reading and writing the `Decimal`
type by transparently serializing/deserializing into the `NUMERIC` data type within PostgreSQL.

### `db-tokio-postgres`

Enables the tokio postgres module allowing for async communication with PostgreSQL.

### `db-diesel-postgres`

Enable `diesel` PostgreSQL support. By default, this enables version `1.4` of `diesel`. If you wish to use the `2.0` 
version of `diesel` then you can do so by using the feature `db-diesel2-postgres`. Please note, if both features are 
enabled then version 2 will supersede version 1.

### `db-diesel-mysql`

Enable `diesel` MySQL support. By default, this enables version `1.4` of `diesel`. If you wish to use the `2.0` 
version of `diesel` then you can do so by using the feature `db-diesel2-mysql`. Please note, if both features are
enabled then version 2 will supersede version 1.

### `legacy-ops`

**Warning:** This is deprecated and will be removed from a future versions.

As of `1.10` the algorithms used to perform basic operations have changed which has benefits of significant speed improvements.
To maintain backwards compatibility this can be opted out of by enabling the `legacy-ops` feature.

### `maths`

The `maths` feature enables additional complex mathematical functions such as `pow`, `ln`, `enf`, `exp` etc.
Documentation detailing the additional functions can be found on the
[`MathematicalOps`](https://docs.rs/rust_decimal/latest/rust_decimal/trait.MathematicalOps.html) trait.  

Please note that `ln` and `log10` will panic on invalid input with `checked_ln` and `checked_log10` the preferred functions
to curb against this. When the `maths` feature was first developed the library would return `0` on invalid input. To re-enable this
non-panicking behavior, please use the feature: `maths-nopanic`.

### `rand`

Implements `rand::distributions::Distribution<Decimal>` to allow the creation of random instances.

Note: When using `rand::Rng` trait to generate a decimal between a range of two other decimals, the scale of the randomly-generated
decimal will be the same as the scale of the input decimals (or, if the inputs have different scales, the higher of the two).

### `rkyv`
Enables [rkyv](https://github.com/rkyv/rkyv) serialization for `Decimal`.
Supports rkyv's safe API when the `rkyv-safe` feature is enabled as well.

### `rocket-traits`

Enable support for Rocket forms by implementing the `FromFormField` trait.

### `rust-fuzz`

Enable `rust-fuzz` support by implementing the `Arbitrary` trait.

### `serde-float`

**Note:** it is recommended to use the `serde-with-*` features for greater control. This allows configurability at the data
level.

Enable this so that JSON serialization of `Decimal` types are sent as a float instead of a string (default).

e.g. with this turned on, JSON serialization would output:
```json
{
  "value": 1.234
}
```

### `serde-str`

**Note:** it is recommended to use the `serde-with-*` features for greater control. This allows configurability at the data
level.

This is typically useful for `bincode` or `csv` like implementations.

Since `bincode` does not specify type information, we need to ensure that a type hint is provided in order to
correctly be able to deserialize. Enabling this feature on its own will force deserialization to use `deserialize_str`
instead of `deserialize_any`.

If, for some reason, you also have `serde-float` enabled then this will use `deserialize_f64` as a type hint. Because
converting to `f64` _loses_ precision, it's highly recommended that you do NOT enable this feature when working with
`bincode`. That being said, this will only use 8 bytes so is slightly more efficient in terms of storage size.

### `serde-arbitrary-precision`

**Note:** it is recommended to use the `serde-with-*` features for greater control. This allows configurability at the data
level.

This is used primarily with `serde_json` and consequently adds it as a "weak dependency". This supports the
`arbitrary_precision` feature inside `serde_json` when parsing decimals.

This is recommended when parsing "float" looking data as it will prevent data loss.

### `serde-with-float`

Enable this to access the module for serializing `Decimal` types to a float. This can be use in `struct` definitions like so:

```rust
#[derive(Serialize, Deserialize)]
pub struct FloatExample {
    #[serde(with = "rust_decimal::serde::float")]
    value: Decimal,
}
```
```rust
#[derive(Serialize, Deserialize)]
pub struct OptionFloatExample {
    #[serde(with = "rust_decimal::serde::float_option")]
    value: Option<Decimal>,
}
```

### `serde-with-str`

Enable this to access the module for serializing `Decimal` types to a `String`. This can be use in `struct` definitions like so:

```rust
#[derive(Serialize, Deserialize)]
pub struct StrExample {
    #[serde(with = "rust_decimal::serde::str")]
    value: Decimal,
}
```
```rust
#[derive(Serialize, Deserialize)]
pub struct OptionStrExample {
    #[serde(with = "rust_decimal::serde::str_option")]
    value: Option<Decimal>,
}
```

### `serde-with-arbitrary-precision`

Enable this to access the module for serializing `Decimal` types to a `String`. This can be use in `struct` definitions like so:

```rust
#[derive(Serialize, Deserialize)]
pub struct ArbitraryExample {
    #[serde(with = "rust_decimal::serde::arbitrary_precision")]
    value: Decimal,
}
```
```rust
#[derive(Serialize, Deserialize)]
pub struct OptionArbitraryExample {
    #[serde(with = "rust_decimal::serde::arbitrary_precision_option")]
    value: Option<Decimal>,
}
```

### `std`

Enable `std` library support. This is enabled by default, however in the future will be opt in. For now, to support `no_std`
libraries, this crate can be compiled with `--no-default-features`.

## Building

Please refer to the [Build document](BUILD.md) for more information on building and testing Rust Decimal.

## Minimum Rust Compiler Version

The current _minimum_ compiler version is [`1.60.0`](https://github.com/rust-lang/rust/blob/master/RELEASES.md#version-1600-2022-04-07)
which was released on `2022-04-07`.

This library maintains support for rust compiler versions that are 4 minor versions away from the current stable rust compiler version.
For example, if the current stable compiler version is `1.50.0` then we will guarantee support up to and including `1.46.0`.
Of note, we will only update the minimum supported version if and when required.
