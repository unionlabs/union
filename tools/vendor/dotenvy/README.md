# dotenvy

[![crates.io](https://img.shields.io/crates/v/dotenvy.svg)](https://crates.io/crates/dotenvy)
[![Released API docs](https://docs.rs/dotenvy/badge.svg)](https://docs.rs/dotenvy)

A well-maintained fork of the [dotenv](https://github.com/dotenv-rs/dotenv) crate.

This crate is the suggested alternative for `dotenv` in security advisory [RUSTSEC-2021-0141](https://rustsec.org/advisories/RUSTSEC-2021-0141.html).

This library loads environment variables from a _.env_ file. This is convenient for dev environments.

The Minimum Supported Rust Version (MSRV) is 1.58.1.

## Components

1. [`dotenvy`](https://crates.io/crates/dotenvy) crate - A well-maintained fork of the `dotenv` crate.
2. [`dotenvy_macro`](https://crates.io/crates/dotenvy_codegen) crate - A macro for compile time dotenv inspection. This is a fork of `dotenv_codegen`.
3. [`dotenvy_codgen_impl`](https://crates.io/crates/dotenvy_codegen_impl) crate - Internal implementation for dotenvy_codegen.
4. `dotenvy` CLI tool for running a command using the environment from a _.env_ file (currently Unix only)

## Usage

### Loading at runtime

```rs
use dotenvy::dotenv;
use std::env;

fn main() {
    dotenv().ok();

    for (key, value) in env::vars() {
        println!("{key}: {value}");
    }
}
```

### Loading at compile time

The `dotenv!` macro provided by `dotenvy_macro` crate can be used.

Warning: there is an outstanding issue with rust-analyzer ([rust-analyzer #9606](https://github.com/rust-analyzer/rust-analyzer/issues/9606)) related to the `dotenv!` macro

## Why does this fork exist?

The original dotenv crate has not been updated since June 26, 2020. Attempts to reach the authors and present maintainer were not successful ([dotenv-rs/dotenv #74](https://github.com/dotenv-rs/dotenv/issues/74)).

This fork intends to serve as the development home for the dotenv implementation in Rust.

## What are the differences from the original?

This repo fixes:

- home directory works correctly (no longer using the deprecated `std::env::home_dir`)
- more helpful errors for `dotenv!` ([dotenv-rs/dotenv #57](https://github.com/dotenv-rs/dotenv/pull/57))

It also adds:

- multiline support for environment variable values
- `io::Read` support via [`from_read`](https://docs.rs/dotenvy/latest/dotenvy/fn.from_read.html) and [`from_read_iter`](https://docs.rs/dotenvy/latest/dotenvy/fn.from_read_iter.html)
- improved docs

For a full list of changes, refer to the [changelog](./CHANGELOG.md).

## The legend

Legend has it that the Lost Maintainer will return, merging changes from `dotenvy` into `dotenv` with such thrust that all `Cargo.toml`s will lose one keystroke. Only then shall the Rust dotenv crateverse be united in true harmony.

Until then, this repo dutifully carries on the dotenv torch. It is actively maintained. Contributions and PRs are very welcome!
