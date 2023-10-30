# `value-bag`

[![Rust](https://github.com/sval-rs/value-bag/workflows/Rust/badge.svg)](https://github.com/sval-rs/value-bag/actions)
[![Latest version](https://img.shields.io/crates/v/value-bag.svg)](https://crates.io/crates/value-bag)
[![Documentation Latest](https://docs.rs/value-bag/badge.svg)](https://docs.rs/value-bag)

## Getting started

Add the `value-bag` crate to your `Cargo.toml`:

```rust
[dependencies.value-bag]
version = "1.0.0-alpha.9"
```

You'll probably also want to add a feature for either `sval` (if you're in a no-std environment) or `serde` (if you need to integrate with other code that uses `serde`):

```rust
[dependencies.value-bag]
version = "1.0.0-alpha.9"
features = ["sval1"]
```

```rust
[dependencies.value-bag]
version = "1.0.0-alpha.9"
features = ["serde1"]
```

Then you're ready to capture anonymous values!

```rust
#[derive(Serialize)]
struct MyValue {
    title: String,
    description: String,
    version: u32,
}

// Capture a value that implements `serde::Serialize`
let bag = ValueBag::capture_serde1(&my_value);

// Print the contents of the value bag
println!("{:?}", bag);
```

## Cargo features

The `value-bag` crate is no-std by default, and offers the following Cargo features:

- `std`: Enable support for the standard library. This allows more types to be captured in a `ValueBag`.
- `error`: Enable support for capturing `std::error::Error`s. Implies `std`.
- `sval`: Enable support for using the [`sval`](https://github.com/sval-rs/sval) serialization framework for inspecting `ValueBag`s by implementing `sval::value::Value`. Implies `sval1`.
    - `sval1`: Enable support for the stable `1.x.x` version of `sval`.
- `serde`: Enable support for using the [`serde`](https://github.com/serde-rs/serde) serialization framework for inspecting `ValueBag`s by implementing `serde::Serialize`. Implies `std` and `serde1`.
    - `serde1`: Enable support for the stable `1.x.x` version of `serde`.
- `test`: Add test helpers for inspecting the shape of the value inside a `ValueBag`.

## What is a value bag?

A `ValueBag` is an anonymous structured bag that supports casting, downcasting, formatting, and serializing. The goal of a `ValueBag` is to decouple the producers of structured data from its consumers. A `ValueBag` can _always_ be interrogated using the consumers serialization API of choice, even if that wasn't the one the producer used to capture the data in the first place.

Say we capture an `i32` using its `Display` implementation as a `ValueBag`:

```rust
let bag = ValueBag::capture_display(42);
```

That value can then be cast to a `u64`:

```rust
let num = bag.as_u64().unwrap();

assert_eq!(42, num);
```

It could also be serialized as a number using `serde`:

```rust
let num = serde_json::to_value(bag).unwrap();

assert!(num.is_number());
```

Say we derive `sval::Value` on a type and capture it as a `ValueBag`:

```rust
#[derive(Value)]
struct Work {
    id: u64,
    description: String,
}

let work = Work {
    id: 123,
    description: String::from("do the work"),
}

let bag = ValueBag::capture_sval1(&work);
```

It could then be formatted using `Display`, even though `Work` never implemented that trait:

```rust
assert_eq!("Work { id: 123, description: \"do the work\" }", bag.to_string());
```

Or serialized using `serde` and retain its nested structure.

The tradeoff in all this is that `ValueBag` needs to depend on the serialization frameworks (`sval`, `serde`, and `std::fmt`) that it supports, instead of just providing an API of its own for others to plug into. Doing this lets `ValueBag` guarantee everything will always line up, and keep its own public API narrow. Each of these frameworks are stable though (except `sval` which is `1.0.0-alpha`).
